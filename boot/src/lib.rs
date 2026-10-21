#![no_main]
#![no_std]


pub mod usb;
pub mod parse;

use defmt_rtt as _; // global logger

use panic_probe as _;
use stm32f4xx_hal as _;


pub mod app{
	use zerocopy::{FromBytes,IntoBytes,Immutable,KnownLayout};
	pub const APP_A_MSG:usize = 0x0802_0000;
	pub const APP_A_START:usize = 0x0802_0000+1024;
	pub const APP_B_MSG:usize = 0x0812_0000;
	pub const APP_B_START:usize = 0x0812_0000+1024;
	pub const APP_MSG_SIZE:usize = size_of::<App>();

	#[derive(Debug,Copy, Clone,FromBytes,IntoBytes,Immutable,KnownLayout)]
	#[repr(C)]
	pub struct App{
		pub version:[u8;4],
		pub size:u32,
		pub sha:[u8;32],
		pub crc32:u32,
	}
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes a semihosting-capable debug tool exit
/// with status code 0.
pub fn exit() -> ! {
    semihosting::process::exit(0);
}

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    semihosting::process::exit(1);
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
