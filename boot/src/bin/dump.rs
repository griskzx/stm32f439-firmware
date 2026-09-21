#![no_main]
#![no_std]

use boot as _; // global logger + panicking-behavior + memory layout
use stm32f4xx_hal::{pac::Peripherals, prelude::*, rcc::Config};
#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    // 配置时钟
    let _rcc = dp.RCC.freeze(Config::hse(25.MHz()));
    boot::exit()
}
