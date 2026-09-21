#![no_main]
#![no_std]

use app as _;
use defmt::info;
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());

    info!("Hello, World!");
    info!("...");
    loop {}
}
