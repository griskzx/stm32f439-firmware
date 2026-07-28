#![no_main]
#![no_std]

use app as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Application started");

    // TODO: Initialize peripherals and run the application.

    app::exit()
}
