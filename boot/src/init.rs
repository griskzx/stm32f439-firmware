use super::*;
use stm32f4xx_hal::{
    pac,
    rcc::{Config, Rcc},
};
pub fn set_clocks(rcc: pac::RCC) -> Rcc {
    let config = Config::hse(8.MHz())
        .hclk(16.MHz())
        .sysclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz())
        .require_pll48clk();
    rcc.freeze(config)
}

use static_cell::{ConstStaticCell, StaticCell};
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

pub type HalUsbBus = UsbBus<USB>;

static EP_MEMORY: ConstStaticCell<[u32; 1024]> = ConstStaticCell::new([0; 1024]);
static USB_BUS: StaticCell<UsbBusAllocator<HalUsbBus>> = StaticCell::new();

pub fn init_usb(
    otg: (pac::OTG_FS_GLOBAL, pac::OTG_FS_DEVICE, pac::OTG_FS_PWRCLK),
    pins: (stm32f4xx_hal::gpio::Pin<'A', 11>, stm32f4xx_hal::gpio::Pin<'A', 12>),
    clocks: &stm32f4xx_hal::rcc::Clocks,
) -> (UsbDevice<'static, HalUsbBus>, SerialPort<'static, HalUsbBus>) {
    let usb = USB::new(otg, pins, clocks);
    let usb_bus = USB_BUS.init(UsbBus::new(usb, EP_MEMORY.take()));

    let serial = SerialPort::new(usb_bus);
    let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("stm32f439-firmware")
            .product("USB echo demo")
            .serial_number("F439-USB-01")])
        .unwrap()
        .build();

    (usb_device, serial)
}

