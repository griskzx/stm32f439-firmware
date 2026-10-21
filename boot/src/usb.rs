//! USB CDC 虚拟串口（中断驱动）：电脑端看到 COM 口，板子端通过两个 SPSC 队列收发字节。
//!
//! 数据流：
//! ```text
//! host -> USB 硬件缓冲 -> [OTG_FS 中断] -> RX 队列 -> 主循环 UsbSerial::read
//! 主循环 UsbSerial::write -> TX 队列 -> [OTG_FS 中断] -> USB 硬件缓冲 -> host
//! ```
//! `usb.poll()`、`serial.read/write/flush` 全部在 OTG_FS 中断里完成，
//! 主循环即使长时间做耗时操作，USB 也不会因为没有及时 poll 而断开。
//! 主循环只接触队列，队列是无锁 SPSC：中断和主循环各持有一端，不需要临界区。
use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free};
use heapless::spsc::{Consumer, Producer, Queue};
use static_cell::{ConstStaticCell, StaticCell};
use stm32f4xx_hal::{
    otg_fs::{USB, UsbBus},
    pac::{self, Interrupt, interrupt},
    prelude::*,
    rcc::Rcc,
};
use usb_device::{UsbError, bus::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

type Bus = UsbBus<USB>;

/// RX / TX 队列大小（字节）。实际可用容量比这个值小 1。
const RX_SIZE: usize = 256;
const TX_SIZE: usize = 512;
/// 一个 USB FS 全速批量包的最大长度。
const PACKET: usize = 64;

// USB 驱动使用的端点缓冲区，共 1024 个 u32，即 4 KiB。
// 它必须一直存在，因此放在 static 中。
static EP_MEMORY: ConstStaticCell<[u32; 1024]> = ConstStaticCell::new([0; 1024]);
// USB 总线分配器要比 UsbDevice / SerialPort 活得更久，中断里要用，所以必须是 'static。
static BUS: StaticCell<UsbBusAllocator<Bus>> = StaticCell::new();
static RX_QUEUE: StaticCell<Queue<u8, RX_SIZE>> = StaticCell::new();
static TX_QUEUE: StaticCell<Queue<u8, TX_SIZE>> = StaticCell::new();

// 中断独占使用的对象：USB 设备、串口类，以及 RX 队列的生产端、TX 队列的消费端。
static ISR_STATE: Mutex<RefCell<Option<IsrState>>> = Mutex::new(RefCell::new(None));

// 电脑用 VID（厂商编号）和 PID（产品编号）识别 USB 设备。
// 这里沿用 HAL 示例中的开发用编号。
const VID: u16 = 0x16c0;
const PID: u16 = 0x27dd;

/// 主循环使用的 USB 串口句柄：只操作 RX / TX 队列，不直接碰 USB 硬件。
pub struct UsbSerial {
    rx: Consumer<'static, u8>,
    tx: Producer<'static, u8>,
}

impl UsbSerial {
    /// 从 RX 队列取出一个字节；队列为空返回 None。
    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        self.rx.dequeue()
    }

    /// 尽量多地从 RX 队列读到 buf，返回读到的字节数（可能为 0）。
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut count = 0;
        while count < buf.len() {
            match self.rx.dequeue() {
                Some(byte) => {
                    buf[count] = byte;
                    count += 1;
                }
                None => break,
            }
        }
        count
    }

    /// 把数据放进 TX 队列并触发一次中断去发送。
    /// 返回实际入队的字节数；队列满时可能小于 data.len()，调用者决定重试还是丢弃。
    pub fn write(&mut self, data: &[u8]) -> usize {
        let mut count = 0;
        for &byte in data {
            if self.tx.enqueue(byte).is_err() {
                break;
            }
            count += 1;
        }
        if count > 0 {
            // 没有 USB 事件时中断不会自己来，手动挂起一次让它把队列里的数据送出去。
            cortex_m::peripheral::NVIC::pend(Interrupt::OTG_FS);
        }
        count
    }

    /// RX 队列中等待读取的字节数。
    #[inline]
    pub fn available(&self) -> usize {
        self.rx.len()
    }
}

/// 只在 OTG_FS 中断中使用的状态。
struct IsrState {
    dev: UsbDevice<'static, Bus>,
    serial: SerialPort<'static, Bus>,
    rx: Producer<'static, u8>,
    tx: Consumer<'static, u8>,
    // 已从 TX 队列取出、但还没被 USB 完全接收的数据。
    // write 可能只接受一部分，所以要记录进度，下次中断接着发。
    pending: [u8; PACKET],
    pos: usize,
    len: usize,
}

impl IsrState {
    fn service(&mut self) {
        // 1. 处理 USB 事件：枚举、控制请求、传输完成。
        self.dev.poll(&mut [&mut self.serial]);

        // 电脑还没完成枚举（或已断开）：丢弃发送方向的数据，避免旧数据在重连后发出。
        if self.dev.state() != UsbDeviceState::Configured {
            self.pos = 0;
            self.len = 0;
            while self.tx.dequeue().is_some() {}
            return;
        }

        // 2. 接收：USB 缓冲 -> RX 队列。
        //    只有 RX 队列至少还能放下一个整包时才读。
        //    否则不读，数据留在 USB 缓冲里，硬件会自动回 NAK，主机会等待重发，不会丢数据。
        let mut buf = [0u8; PACKET];
        while self.rx.capacity() - self.rx.len() >= PACKET {
            match self.serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    for &byte in &buf[..count] {
                        let _ = self.rx.enqueue(byte);
                    }
                }
                Ok(_) | Err(UsbError::WouldBlock) => break,
                Err(_) => break,
            }
        }

        // 3. 发送：TX 队列 -> USB 缓冲。
        loop {
            if self.pos == self.len {
                // 上一批已经发完，从队列里再取一批。
                self.pos = 0;
                self.len = 0;
                while self.len < PACKET {
                    match self.tx.dequeue() {
                        Some(byte) => {
                            self.pending[self.len] = byte;
                            self.len += 1;
                        }
                        None => break,
                    }
                }
                if self.len == 0 {
                    break;
                }
            }
            match self.serial.write(&self.pending[self.pos..self.len]) {
                Ok(count) if count > 0 => self.pos += count,
                // USB 缓冲暂时满了：保留 pending，等下一次传输完成中断再继续。
                Ok(_) | Err(UsbError::WouldBlock) => break,
                Err(_) => {
                    self.pos = 0;
                    self.len = 0;
                    break;
                }
            }
        }

        // 4. 把 serial 内部缓冲里不足一个包的数据也推出去。
        let _ = self.serial.flush();
    }
}

/// 初始化 USB 并开启 OTG_FS 中断，然后立即返回；之后 USB 由中断驱动。
/// 返回的 UsbSerial 给主循环收发数据。
pub fn init(
    // GPIOA 包含 USB 使用的 PA11、PA12 引脚。
    gpioa: pac::GPIOA,
    // 同一个 USB OTG FS 外设的三组寄存器，由 main.rs 转交所有权。
    global: pac::OTG_FS_GLOBAL,
    device: pac::OTG_FS_DEVICE,
    pwrclk: pac::OTG_FS_PWRCLK,
    rcc: &mut Rcc,
) -> UsbSerial {
    // USB FS 需要准确的 48 MHz 时钟；main.rs 中 require_pll48clk() 负责配置。
    assert!(rcc.clocks.is_pll48clk_valid());
    // 把 GPIOA 拆分成可以分别配置的引脚。
    let gpioa = gpioa.split(rcc);

    // 先把 D+ 拉低约 10 ms，模拟 USB 断开，方便复位后让电脑重新识别设备。
    // 这只发生在初始化阶段，此时中断还没有开启。
    let mut dp = gpioa.pa12.into_push_pull_output();
    dp.set_low();
    cortex_m::asm::delay(rcc.clocks.sysclk().to_Hz() / 100);
    // HAL 接管 USB 寄存器、PA11(D−)、PA12(D+)，并设置引脚复用。
    let usb = USB::new((global, device, pwrclk), (gpioa.pa11, dp), &rcc.clocks);
    let bus: &'static UsbBusAllocator<Bus> = BUS.init(UsbBus::new(usb, EP_MEMORY.take()));
    let serial = SerialPort::new(bus);

    // 每颗芯片有 96 位唯一编号。把它转成 24 个十六进制字符，作为 USB 序列号。
    // 序列号字符串要一直有效，所以放进 static。
    static SERIAL_NUMBER: StaticCell<[u8; 24]> = StaticCell::new();
    let serial_number = SERIAL_NUMBER.init([0; 24]);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for (index, &word) in read_uid().iter().enumerate() {
        for digit in 0..8 {
            // 从高位到低位，每次取 4 bit，映射为一个十六进制字符。
            serial_number[index * 8 + digit] = HEX[((word >> ((7 - digit) * 4)) & 15) as usize];
        }
    }
    // 上面只写入了 ASCII 十六进制字符，因此这里一定是合法的 UTF-8。
    let serial_number: &'static str = core::str::from_utf8(serial_number).unwrap();

    // USB 设备描述符：电脑枚举设备时会读取这些信息。
    let dev = UsbDeviceBuilder::new(bus, UsbVidPid(VID, PID))
        .device_class(usbd_serial::USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("STM32")
            .product("STM32F439 USB CDC")
            .serial_number(serial_number)])
        .unwrap()
        .build();

    // 两个队列各自拆成两端：一端给中断，一端给主循环。
    let (rx_prod, rx_cons) = RX_QUEUE.init(Queue::new()).split();
    let (tx_prod, tx_cons) = TX_QUEUE.init(Queue::new()).split();

    free(|cs| {
        ISR_STATE.borrow(cs).replace(Some(IsrState {
            dev,
            serial,
            rx: rx_prod,
            tx: tx_cons,
            pending: [0; PACKET],
            pos: 0,
            len: 0,
        }));
    });
    // 状态放好之后才开中断。
    unsafe { cortex_m::peripheral::NVIC::unmask(Interrupt::OTG_FS) };

    defmt::info!("USB CDC ready on PA11/PA12 (interrupt driven)");

    UsbSerial {
        rx: rx_cons,
        tx: tx_prod,
    }
}

#[interrupt]
fn OTG_FS() {
    // 中断里不会被同级中断打断，这里的临界区只是为了安全地借用 static。
    free(|cs| {
        if let Some(state) = ISR_STATE.borrow(cs).borrow_mut().as_mut() {
            state.service();
        }
    });
}

fn read_uid() -> [u32; 3] {
    // STM32F439 出厂写入的 96 位 UID，从这个固定地址开始，占三个 u32。
    let uid = 0x1fff_7a10 as *const u32;
    // 裸指针读取需要 unsafe。这里只访问芯片手册规定的只读 UID 地址。
    // read_volatile 要求实际读取寄存器内容；add(1)、add(2) 各偏移一个 u32。
    unsafe {
        [
            core::ptr::read_volatile(uid),
            core::ptr::read_volatile(uid.add(1)),
            core::ptr::read_volatile(uid.add(2)),
        ]
    }
}
