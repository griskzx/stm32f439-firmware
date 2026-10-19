//! Polled USB CDC transport. Keep polling even while the host is not reading.
use cortex_m::peripheral::{DWT, SCB};
use static_cell::ConstStaticCell;
use stm32f4xx_hal::{
    otg_fs::{USB, UsbBus},
    pac,
    prelude::*,
    rcc::Rcc,
};
use usb_device::{UsbError, prelude::*};

use crate::cli::{Action, BoardInfo, Console, SLOT_A, SLOT_B};

static EP_MEMORY: ConstStaticCell<[u32; 1024]> = ConstStaticCell::new([0; 1024]);

// Development-only VID/PID, matching the HAL USB serial example.
const VID: u16 = 0x16c0;
const PID: u16 = 0x27dd;

pub fn run(
    gpioa: pac::GPIOA,
    global: pac::OTG_FS_GLOBAL,
    device: pac::OTG_FS_DEVICE,
    pwrclk: pac::OTG_FS_PWRCLK,
    rcc: &mut Rcc,
) -> ! {
    assert!(rcc.clocks.is_pll48clk_valid());
    let mut cp = cortex_m::Peripherals::take().unwrap();
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    let gpioa = gpioa.split(rcc);
    // Force D+ low briefly so firmware restarts also cause re-enumeration.
    let mut dp = gpioa.pa12.into_push_pull_output();
    dp.set_low();
    cortex_m::asm::delay(rcc.clocks.sysclk().to_Hz() / 100);
    let usb = USB::new((global, device, pwrclk), (gpioa.pa11, dp), &rcc.clocks);
    let bus = UsbBus::new(usb, EP_MEMORY.take());
    let mut serial = usbd_serial::SerialPort::new(&bus);

    let uid = read_uid();
    let mut serial_number = [0; 24];
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for (index, &word) in uid.iter().enumerate() {
        for digit in 0..8 {
            serial_number[index * 8 + digit] = HEX[((word >> ((7 - digit) * 4)) & 15) as usize];
        }
    }
    let serial_number = core::str::from_utf8(&serial_number).unwrap();
    let mut usb = UsbDeviceBuilder::new(&bus, UsbVidPid(VID, PID))
        .device_class(usbd_serial::USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("STM32 demo")
            .product("STM32F439 Boot CLI")
            .serial_number(serial_number)])
        .unwrap()
        .build();

    let info = BoardInfo {
        version: env!("CARGO_PKG_VERSION"),
        sysclk: rcc.clocks.sysclk().to_Hz(),
        hclk: rcc.clocks.hclk().to_Hz(),
        pclk1: rcc.clocks.pclk1().to_Hz(),
        pclk2: rcc.clocks.pclk2().to_Hz(),
        usbclk: rcc.clocks.pll48clk().unwrap().to_Hz(),
        uid,
        vectors: [read_vectors(SLOT_A), read_vectors(SLOT_B)],
    };
    let mut console = Console::new();
    let mut rx = [0; 64];
    let mut rx_len = 0;
    let mut rx_pos = 0;
    let mut opened = false;
    let mut reboot_started: Option<u32> = None;
    defmt::info!("Boot: USB CDC ready on PA11/PA12; open port with DTR enabled");

    loop {
        usb.poll(&mut [&mut serial]);

        // Give the response a bounded 250 ms to reach the host. A disconnected
        // or stalled host must never prevent a requested reset.
        if let Some(start) = reboot_started
            && DWT::cycle_count().wrapping_sub(start) >= info.sysclk / 4
        {
            SCB::sys_reset();
        }

        let connected = usb.state() == UsbDeviceState::Configured && serial.dtr();
        if !connected {
            opened = false;
            rx_pos = 0;
            rx_len = 0;
            // Discard bytes sent with the port closed, including partial commands.
            let _ = serial.read(&mut rx);
            continue;
        }
        if !opened {
            console.open();
            opened = true;
        }

        if !console.pending().is_empty() {
            match serial.write(console.pending()) {
                Ok(count) => console.consume(count),
                Err(UsbError::WouldBlock) => {}
                Err(error) => defmt::warn!("USB write error: {:?}", defmt::Debug2Format(&error)),
            }
        }
        // Flush short packets too; WouldBlock is normal until the host reads.
        let _ = serial.flush();
        if reboot_started.is_some() || !console.pending().is_empty() {
            continue;
        }

        if rx_pos == rx_len {
            match serial.read(&mut rx) {
                Ok(count) => {
                    rx_pos = 0;
                    rx_len = count;
                }
                Err(UsbError::WouldBlock) => continue,
                Err(error) => {
                    defmt::warn!("USB read error: {:?}", defmt::Debug2Format(&error));
                    continue;
                }
            }
        }
        if rx_pos < rx_len {
            // Retain the rest of this USB packet while the response is draining.
            let action = console.receive(rx[rx_pos], &info);
            rx_pos += 1;
            if action == Action::Reboot {
                reboot_started = Some(DWT::cycle_count());
            }
        }
    }
}

fn read_uid() -> [u32; 3] {
    // STM32F439's factory-programmed 96-bit UID, three aligned read-only words.
    let uid = 0x1fff_7a10 as *const u32;
    unsafe {
        [
            core::ptr::read_volatile(uid),
            core::ptr::read_volatile(uid.add(1)),
            core::ptr::read_volatile(uid.add(2)),
        ]
    }
}

fn read_vectors(base: u32) -> [u32; 2] {
    // Called only for the two fixed, aligned slot addresses inside 2 MiB Flash.
    let vectors = base as *const u32;
    unsafe {
        [
            core::ptr::read_volatile(vectors),
            core::ptr::read_volatile(vectors.add(1)),
        ]
    }
}
