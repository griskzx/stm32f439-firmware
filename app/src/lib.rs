#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application through a semihosting-capable debugger.
pub fn exit() -> ! {
    semihosting::process::exit(0);
}

#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    semihosting::process::exit(1);
}

#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
