//! USB CDC 虚拟串口：电脑端看到 COM 口，板子端通过 serial 收发字节。
//!
//! main.rs 配置好时钟后调用 run()，程序随后一直在本文件的 loop 中运行。
//! 下方“接收”和“发送”两处就是以后编写通信逻辑的位置。
use static_cell::ConstStaticCell;
use stm32f4xx_hal::{
    otg_fs::{USB, UsbBus},
    pac,
    prelude::*,
    rcc::Rcc,
};
use usb_device::{UsbError, prelude::*};

// USB 驱动使用的端点缓冲区，共 1024 个 u32，即 4 KiB。
// 它必须一直存在，因此放在 static 中；它和下面应用层的 buf 不是同一块内存。
// ConstStaticCell 可以安全地取出一次可变引用，不需要使用 static mut。
static EP_MEMORY: ConstStaticCell<[u32; 1024]> = ConstStaticCell::new([0; 1024]);

// 电脑用 VID（厂商编号）和 PID（产品编号）识别 USB 设备。
// 这里沿用 HAL 示例中的开发用编号。
const VID: u16 = 0x16c0;
const PID: u16 = 0x27dd;

/// 初始化 USB，并运行原始字节回显示例。返回类型 ! 表示此函数不会返回。
pub fn run(
    // GPIOA 包含 USB 使用的 PA11、PA12 引脚。
    gpioa: pac::GPIOA,
    // 同一个 USB OTG FS 外设的三组寄存器，由 main.rs 转交所有权。
    global: pac::OTG_FS_GLOBAL,
    device: pac::OTG_FS_DEVICE,
    pwrclk: pac::OTG_FS_PWRCLK,
    rcc: &mut Rcc,
) -> ! {
    // USB FS 需要准确的 48 MHz 时钟；main.rs 中 require_pll48clk() 负责配置。
    assert!(rcc.clocks.is_pll48clk_valid());
    // 把 GPIOA 拆分成可以分别配置的引脚。
    let gpioa = gpioa.split(rcc);

    // 先把 D+ 拉低约 10 ms，模拟 USB 断开，方便复位后让电脑重新识别设备。
    // 这里只是初始化时短暂延时，正常通信循环中不能这样长时间阻塞。
    let mut dp = gpioa.pa12.into_push_pull_output();
    dp.set_low();
    cortex_m::asm::delay(rcc.clocks.sysclk().to_Hz() / 100);
    // HAL 接管 USB 寄存器、PA11(D−)、PA12(D+)，并设置引脚复用。
    let usb = USB::new((global, device, pwrclk), (gpioa.pa11, dp), &rcc.clocks);
    // bus 分配 USB 端点和缓冲区；它要比 serial 和 USB 设备对象活得更久。
    let bus = UsbBus::new(usb, EP_MEMORY.take());
    // serial 是 CDC 串口接口：后面用它的 read / write / flush 收发数据。
    let mut serial = usbd_serial::SerialPort::new(&bus);

    // 每颗芯片有 96 位唯一编号。把它转成 24 个十六进制字符，作为 USB 序列号。
    // 例如一个 u32 的 0x1234ABCD 会转成字符串 "1234ABCD"。
    let uid = read_uid();
    let mut serial_number = [0; 24];
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for (index, &word) in uid.iter().enumerate() {
        for digit in 0..8 {
            // 从高位到低位，每次取 4 bit，映射为一个十六进制字符。
            serial_number[index * 8 + digit] = HEX[((word >> ((7 - digit) * 4)) & 15) as usize];
        }
    }
    // 上面只写入了 ASCII 十六进制字符，因此这里一定是合法的 UTF-8。
    let serial_number = core::str::from_utf8(&serial_number).unwrap();

    // USB 设备描述符：电脑枚举设备时会读取这些信息。
    // CDC 表示虚拟串口；字符串对应厂商名、产品名和序列号。
    let mut usb = UsbDeviceBuilder::new(&bus, UsbVidPid(VID, PID))
        .device_class(usbd_serial::USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("STM32")
            .product("STM32F439 USB CDC")
            .serial_number(serial_number)])
        .unwrap()
        .build();

    defmt::info!("USB CDC ready on PA11/PA12");

    // ===== 示例：电脑发来什么字节，板子就原样发回什么字节 =====
    // 只创建一次缓冲区，放在 loop 外，避免下一轮循环丢失尚未发送的数据。
    let mut buf = [0u8; 64];
    let mut received = 0; // buf 中实际有效的数据长度。
    let mut sent = 0; // 其中已经交给 USB 发送缓冲区的字节数。

    loop {
        // 1. 每一轮都要处理 USB 事件，例如枚举、控制请求和传输完成。
        //    即使暂时没有数据，或者发送忙，也必须继续 poll。
        usb.poll(&mut [&mut serial]);

        // 电脑还没完成枚举时不收发，同时清除上次连接剩余的应用层数据。
        if usb.state() != UsbDeviceState::Configured {
            received = 0;
            sent = 0;
            continue;
        }

        // 2. 接收：电脑 -> 板子。
        //    等上一批数据全部交给发送缓冲区后，再接收下一批，防止覆盖 buf。
        if sent == received {
            match serial.read(&mut buf) {
                Ok(count) => {
                    received = count;
                    sent = 0;

                    // 【你自己的接收处理写在这里】
                    // 本次收到的是 &buf[..received]，其余部分不是本次有效数据。
                    // 当前示例不做解析，下面会把这批字节原样发回电脑。
                    // 注意：一次 read 不等于一条完整命令，命令可能分多次收到。
                }
                Err(UsbError::WouldBlock) => {
                    // 暂时没有数据，这是正常情况；下一轮继续 poll 和 read。
                }
                Err(error) => {
                    defmt::warn!("USB read error: {:?}", defmt::Debug2Format(&error));
                }
            }
        }

        // 3. 发送：板子 -> 电脑。
        //    【你自己的回复在这里发送】当前示例发送收到的原始数据。
        if sent < received {
            match serial.write(&buf[sent..received]) {
                Ok(count) => {
                    // write 可能只接受一部分字节，必须记录进度，下次接着发送。
                    // 返回成功只代表已交给驱动，不代表电脑程序已经读到了。
                    sent += count;
                }
                Err(UsbError::WouldBlock) => {
                    // 发送缓冲区暂时满了。保留 buf 和 sent，下一轮再试。
                    // 不要在这里死等，否则 USB 事件无法得到处理。
                }
                Err(error) => {
                    // 非 WouldBlock 错误：记录错误并放弃本批回显。
                    defmt::warn!("USB write error: {:?}", defmt::Debug2Format(&error));
                    received = 0;
                    sent = 0;
                }
            }
        }

        // 4. 推动驱动中尚未发完的数据继续发送，包括不足一个 USB 包的数据。
        //    flush 也是非阻塞的：WouldBlock 表示还没发完，下一轮再调用即可。
        //    即使 flush 成功，也不表示电脑上的串口工具已经读取了数据。
        match serial.flush() {
            Ok(()) | Err(UsbError::WouldBlock) => {}
            Err(error) => {
                defmt::warn!("USB flush error: {:?}", defmt::Debug2Format(&error));
            }
        }
    }
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
