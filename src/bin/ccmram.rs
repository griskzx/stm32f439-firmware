#![no_main]
#![no_std]

use bootloader as _;

#[unsafe(link_section = ".ccmram.buffers")]
static mut FAST_BUFFER: [u32; 1024] = [0; 1024];

#[cortex_m_rt::entry]
fn main() -> ! {
    let buffer = core::ptr::addr_of_mut!(FAST_BUFFER);

    unsafe {
        (*buffer)[0] = 0x1234_5678;
        (*buffer)[1023] = 0x8765_4321;
    }

    let address = buffer as usize;
    defmt::info!("FAST_BUFFER is at {=usize:#010x}", address);
    defmt::assert!(address >= 0x1000_0000 && address < 0x1001_0000);

    bootloader::exit()
}
