#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::Timer;
use panic_probe as _;

#[unsafe(link_section = ".ccm")]
static mut CCM_BUFFER: MaybeUninit<[u32; 256]> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let _p = embassy_stm32::init(config);

    // SAFETY: this task owns the buffer exclusively; no other task or interrupt accesses it.
    // The NOLOAD section is initialized in full before creating a reference.
    let buffer = unsafe {
        let ptr = (&raw mut CCM_BUFFER).cast::<[u32; 256]>();
        ptr.write_bytes(0, 1);
        &mut *ptr
    };
    // Volatile accesses keep the demonstration observable in optimized builds.
    unsafe { buffer.as_mut_ptr().write_volatile(439) };
    let value = unsafe { buffer.as_ptr().read_volatile() };
    info!(
        "CCM buffer: address={=usize:#x}, value={=u32}",
        buffer.as_ptr() as usize,
        value
    );

    loop {
        info!("Hello World!");
        Timer::after_secs(1).await;
    }
}
