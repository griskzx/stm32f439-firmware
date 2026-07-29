#![no_std]
#![no_main]

use bootloader as _;
use bootloader::boot::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    unsafe {
        jump_to_app(APP_A);
    }
}
