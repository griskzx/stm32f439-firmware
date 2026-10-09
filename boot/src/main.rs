#![no_std]
#![no_main]

use boot as _;
use core::mem::MaybeUninit;
use stm32f4xx_hal::pac;

#[unsafe(link_section = ".ccm")]
static mut CCM_BUFFER: MaybeUninit<[u32; 256]> = MaybeUninit::uninit();

#[cortex_m_rt::entry]
fn main() -> ! {
    let _peripherals = pac::Peripherals::take().unwrap();
    defmt::info!("Boot initialized");

    // SAFETY: main owns this buffer exclusively; no interrupt accesses it.
    // The NOLOAD section is initialized in full before creating a reference.
    let buffer = unsafe {
        let ptr = (&raw mut CCM_BUFFER).cast::<[u32; 256]>();
        ptr.write_bytes(0, 1);
        &mut *ptr
    };
    // Volatile accesses keep the demonstration observable in optimized builds.
    unsafe { buffer.as_mut_ptr().write_volatile(439) };
    let value = unsafe { buffer.as_ptr().read_volatile() };
    defmt::info!(
        "CCM buffer: address={=usize:#x}, value={=u32}",
        buffer.as_ptr() as usize,
        value
    );

    loop {
        cortex_m::asm::wfi();
    }
}
