#![no_std]
#![no_main]

use stm32f4xx_hal::{
    pac,
    flash::{Error as FlashError,FlashExt}
};

const METADATA_SECTOR:u8 = 23;
const METADATA_OFFSET:usize = 0x001E_0000;
const METADATA_SIZE:usize = 128*1024;
#[cortex_m_rt::entry]
fn main()->! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH;

    let data = b"hello world";

    write_flash(&mut flash,data).unwrap();

    let read_back = read_flash(&flash,data.len());

    let text = core::str::from_utf8(read_back).unwrap();

    defmt::info!("Read back: {:?}", text);

    bootloader::exit()
}

// 擦除后在写
fn write_flash(flash:&mut pac::FLASH,data:&[u8])->Result<(),FlashError>{
    assert!(data.len() <= METADATA_SIZE);

    //解锁flash
    let mut unlocked = flash.unlocked();

    // 擦除整个扇区
    unlocked.erase(METADATA_SECTOR)?;

    // 参数相对于0x0800_0000,而且绝对地址
    unlocked.program(METADATA_OFFSET,data.iter())?;

    Ok(())
}

fn read_flash(flash: &pac::FLASH, len:usize) -> &[u8] {
    assert!(len <=  METADATA_SIZE);
    &flash.read()[METADATA_OFFSET..METADATA_OFFSET+len]
}