#![no_main]
#![no_std]

use boot as _;
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

    let mut buffer = [0u8; 64];
    let mut received = 0;
    let mut sent = 0;

    defmt::info!("USB echo demo started");

    loop {
        let usb_event = usb_device.poll(&mut [&mut serial]);

        // 先把上一包数据完整回传，再接收下一包，避免丢数据。
        if sent < received {
            if let Ok(count) = serial.write(&buffer[sent..received]) {
                sent += count;
            }
            continue;
        }

        if usb_event {
            if let Ok(count) = serial.read(&mut buffer) {
                received = count;
                sent = 0;
            }
        }
    }
}
