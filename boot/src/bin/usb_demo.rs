#![no_main]
#![no_std]

use boot as _;
use heapless::Deque;
use static_cell::ConstStaticCell;
use stm32f4xx_hal::{
    otg_fs::{USB, UsbBus},
    pac,
    prelude::*,
    rcc::Config,
};
use usb_device::prelude::{StringDescriptors, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

static EP_MEMORY: ConstStaticCell<[u32; 1024]> = ConstStaticCell::new([0; 1024]);

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // NUCLEO-F439ZI 默认由 ST-LINK MCO 向 OSC_IN 提供 8 MHz 时钟。
    // USB OTG FS 需要精确的 48 MHz PLL 时钟。
    let mut rcc = dp.RCC.freeze(
        Config::hse(8.MHz())
            .bypass_hse_oscillator()
            .sysclk(168.MHz())
            .require_pll48clk(),
    );
    let gpioa = dp.GPIOA.split(&mut rcc);

    // PA11 = USB D-, PA12 = USB D+。
    let usb = USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &rcc.clocks,
    );
    let usb_bus = UsbBus::new(usb, EP_MEMORY.take());

    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("stm32f439-firmware")
            .product("USB echo demo")
            .serial_number("F439-USB-01")])
        .unwrap()
        .build();

    let mut tx_queue = Deque::<u8, 128>::new();
    let mut rx_buf = [0u8; 128];

    defmt::info!("USB echo demo started");

    loop {
        let usb_event = usb_device.poll(&mut [&mut serial]);

        if !tx_queue.is_empty() {
            defmt::debug!("正在发送数据");
            let (slice1, _) = tx_queue.as_slices();
            if let Ok(count) = serial.write(slice1) {
                for _ in 0..count {
                    tx_queue.pop_front();
                }
            }
        }

        if usb_event {
            if let Ok(count) = serial.read(&mut rx_buf) {
                for &byte in &rx_buf[..count] {
                    defmt::debug!("接收到字节: {:#04x}", byte);
                    defmt::debug!("接收到字节: {}", byte as char);
                    let _ = tx_queue.push_back(byte);
                }
            }
        }
    }
}
