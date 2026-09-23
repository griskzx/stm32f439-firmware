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
