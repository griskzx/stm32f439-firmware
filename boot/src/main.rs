//! 共2m flash，
//! 第一个1m 前128kb boot
//! 后896kb为app a  Sector 5    0x0802_0000
//! 第二个1m 前128kb data Sector 12  0x0810_0000
//! 后896kb为app b  Sector 17  0x0812_0000

#![no_std]
#![no_main]

use boot as _;
use boot::init::{init_usb, set_clocks};
use boot::msg::{BOOT_CONTEXT_MAGIC, Context, Partition};
use defmt::{info, warn};
use fsapos as _;
use heapless::{Deque, Vec};
use stm32f4xx_hal::flash::FlashExt;
use stm32f4xx_hal::prelude::*;

const ADDR_BASE: usize = 0x08000_0000;
const BOOT_CONTEXT_ADDR: usize = 0x08002_0000;

const SECTOR_A_NUM: u8 = 12;
const APP_A: usize = 0x810_0000;
const APP_A_FIRMWARE: usize = 0x810_0000 + 1024;

const SECTOR_B_NUM: u8 = 20;
const APP_B: usize = 0x818_0000;
const APP_B_FIRMWARE: usize = 0x818_0000 + 1024;

use APP_A as APP_A_MSG;
use APP_B as APP_B_MSG;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();

    info!("set clocks");
    let mut rcc = set_clocks(dp.RCC);
    let gpioa = dp.GPIOA.split(&mut rcc);

    let (mut usb_device, mut serial) = init_usb(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &rcc.clocks,
    );

    let mut tx_queue = Deque::<u8, 1024>::new();

    let mut rx_buf = Vec::<u8, 1024>::new();
    rx_buf.resize_default(1024).unwrap();

    let mut write_buf = Deque::<u8, 1024>::new();

    info!("USB echo started in main");

    let context_magic =
        unsafe { core::ptr::read_unaligned(BOOT_CONTEXT_ADDR as *const u8 as *const u32) };
    let mut context = if context_magic != BOOT_CONTEXT_MAGIC {
        Context::new()
    } else {
        unsafe { core::ptr::read_unaligned(BOOT_CONTEXT_ADDR as *const u8 as *const Context) }
    };

    let app_addr = if context.valid() == Partition::A {
        (APP_B, SECTOR_B_NUM)
    } else {
        (APP_A, SECTOR_A_NUM)
    };

    // 解锁flash控制器
    info!("解锁flash...");
    let mut flash_unlocked = dp.FLASH.unlocked();

    //擦除对应块
    info!("擦除扇区:{}", app_addr.1);
    //此处假设擦除所有app块
    if let Err(_) = flash_unlocked.erase(app_addr.1) {
        warn!("檫除对应扇区失败");
        tx_queue.extend(b"Erasing the corresponding sector failed!!!");
    }

    loop {
        let usb_event = usb_device.poll(&mut [&mut serial]);

        if !tx_queue.is_empty() {
            let (slice1, _) = tx_queue.as_slices();
            if let Ok(count) = serial.write(slice1) {
                for _ in 0..count {
                    tx_queue.pop_front();
                }
            }
        }

        if usb_event {
            //缓冲区满了直接抛弃掉
            if let Ok(_count) = serial.read(rx_buf.as_mut_slice()) {}
        }
        if rx_buf.is_full() {
            while !rx_buf.is_empty() {
                //队列从头开始push rx栈pop，此时队列的数据为正序
                write_buf.push_front(rx_buf.pop().unwrap()).unwrap();
            }
        }
        if write_buf.is_full() {
            //此处判断帧类型是app数据还是app固件
            //然后写入对应flash
        }
    }
}
