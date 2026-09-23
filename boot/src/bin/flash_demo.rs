#![no_main]
#![no_std]

use boot::{self as _, exit, init::set_clocks};
use stm32f4xx_hal::{flash::FlashExt, pac::Peripherals};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dp = Peripherals::take().unwrap();
    let _rcc = set_clocks(dp.RCC);

    defmt::info!("=== STM32F439 Flash Demo Started ===");

    // 目标扇区：Sector 5（位于 128KB boot 分区之后，起始物理地址 0x0802_0000）
    const SECTOR_NUM: u8 = 5;
    const TARGET_OFFSET: usize = 128 * 1024; // 相对 Flash 基地址 0x0800_0000 的偏移（128KB）
    const TARGET_ADDR: *const u8 = (0x0800_0000 + TARGET_OFFSET) as *const u8;
    const TEST_DATA: &[u8] = b"Hello STM32F4 Flash from Rust Bootloader!";

    // 1. 解锁 Flash 控制器
    defmt::info!("1. 解锁 Flash 控制器...");
    let mut unlocked = dp.FLASH.unlocked();

    // 2. 擦除扇区 5
    defmt::info!(
        "2. 擦除扇区 {} (物理地址 0x{:08x})...",
        SECTOR_NUM,
        TARGET_ADDR as usize
    );
    unlocked.erase(SECTOR_NUM).expect("Flash 扇区擦除失败");

    // 3. 写入测试数据
    defmt::info!("3. 写入测试数据 ({} 字节)...", TEST_DATA.len());
    unlocked
        .program(TARGET_OFFSET, TEST_DATA.iter())
        .expect("Flash 写入失败");

    // 4. 显式释放 unlocked，触发 Drop 自动重新上锁
    drop(unlocked);
    defmt::info!("4. Flash 已重新上锁。");

    // 5. 校验读取（直接通过内存映射指针读取）
    let read_back = unsafe { core::slice::from_raw_parts(TARGET_ADDR, TEST_DATA.len()) };
    if read_back == TEST_DATA {
        if let Ok(s) = core::str::from_utf8(read_back) {
            defmt::info!("5. 校验成功，读取数据: '{}'", s);
        } else {
            defmt::info!("5. 校验成功！");
        }
    } else {
        defmt::error!("5. 校验失败！数据不匹配。");
        defmt::error!("期望值: {:?}", TEST_DATA);
        defmt::error!("读取值: {:?}", read_back);
    }

    defmt::info!("=== STM32F439 Flash Demo Finished ===");
    exit()
}
