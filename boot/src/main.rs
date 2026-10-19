#![no_main]
#![no_std]

use boot as _; // global logger + panicking-behavior + memory layout
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};

mod cli;
mod usb_cli;

// NUCLEO-F439ZI (MB1137): ST-LINK's STM32F103 supplies an 8 MHz MCO
// to PH0/OSC_IN. Use HSE bypass with the board's default MCO solder bridges.
const HSE_HZ: u32 = 8_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    defmt::info!(
        "Boot: initializing HSE bypass from ST-LINK MCO ({} Hz)",
        HSE_HZ
    );

    // 168 MHz runs without OverDrive. HAL configures the PLL, Flash wait
    // states and bus prescalers, and waits for HSE and PLL readiness.
    let mut rcc = dp.RCC.freeze(
        Config::hse(HSE_HZ.Hz())
            .bypass_hse_oscillator()
            .sysclk(168.MHz())
            .hclk(168.MHz())
            .pclk1(42.MHz())
            .pclk2(84.MHz())
            .require_pll48clk(),
    );

    // Confirm the hardware has switched SYSCLK to PLL before reporting it.
    while !rcc.cfgr().read().sws().is_pll() {}

    defmt::info!("Boot: clock source = HSE bypass -> PLL");
    defmt::info!(
        "Boot: SYSCLK={} Hz, HCLK={} Hz, PCLK1={} Hz, PCLK2={} Hz",
        rcc.clocks.sysclk().to_Hz(),
        rcc.clocks.hclk().to_Hz(),
        rcc.clocks.pclk1().to_Hz(),
        rcc.clocks.pclk2().to_Hz(),
    );

    usb_cli::run(
        dp.GPIOA,
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        &mut rcc,
    )
}
