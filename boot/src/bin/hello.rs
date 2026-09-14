#![no_main]
#![no_std]

use boot::{self as _, init::set_clocks};
use stm32f4xx_hal::{
    gpio::GpioExt, hal_02::blocking::delay::DelayMs, pac::Peripherals, timer::SysTimerExt,
}; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut rcc = set_clocks(dp.RCC);
    let gpiob = dp.GPIOB.split(&mut rcc);
    // 推挽输出
    let mut green_led = gpiob.pb0.into_push_pull_output();
    let mut delay = cp.SYST.delay(&rcc.clocks);
    defmt::info!("Hello, world!");
    loop {
        green_led.set_high();
        delay.delay_ms(500u32);
        green_led.set_low();
        delay.delay_ms(500u32);
    }
}
