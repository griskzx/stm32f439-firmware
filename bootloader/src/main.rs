#![no_std]
#![no_main]

mod led;

use bootloader as _;
use cortex_m::delay::Delay;
use stm32f4xx_hal::{pac, prelude::*, rcc};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let _clocks = rcc::Config::hse(8.MHz())
        .sysclk(168.MHz())
        .hclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz())
        .i2s_clk(48.MHz());
    let mut clocks = rcc.freeze(_clocks);

    // 获取时钟
    let mut delay = Delay::new(cp.SYST, 168_000_000);

    let gpiob = dp.GPIOB.split(&mut clocks);

    let gpiob_pin0 = gpiob.pb0.into_push_pull_output();
    let gpiob_pin7 = gpiob.pb7.into_push_pull_output();
    let gpiob_pin14 = gpiob.pb14.into_push_pull_output();

    let mut green_led = led::Led::new(gpiob_pin0);
    let mut blue_led = led::Led::new(gpiob_pin7);
    let mut red_led = led::Led::new(gpiob_pin14);

    loop {
        green_led.on().unwrap();
        blue_led.on().unwrap();
        red_led.on().unwrap();

        delay.delay_ms(500);

        green_led.off().unwrap();
        blue_led.off().unwrap();
        red_led.off().unwrap();
        delay.delay_ms(500);
    }
    // bootloader::exit()
}
