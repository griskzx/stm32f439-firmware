use cortex_m::peripheral::SCB;
use cortex_m::{asm, interrupt};
use stm32f4xx_hal::pac::CorePeripherals;

pub unsafe fn dump(addr: usize) -> ! {
    //     let initial_msp = unsafe { core::ptr::read_volatile(addr as *const u32) };
    //     let reset_map = unsafe { core::ptr::read_volatile((addr + 4) as *const u32) };

    //关闭中断
    interrupt::disable();

    let mut cp = unsafe { CorePeripherals::steal() };

    cp.SYST.disable_interrupt();
    cp.SYST.disable_counter();

    SCB::clear_pendst();
    SCB::clear_pendsv();

    for bank in 0..3 {
        unsafe {
            cp.NVIC.icer[bank].write(u32::MAX);
            cp.NVIC.icpr[bank].write(u32::MAX);
        }
    }
    unsafe {
        cp.SCB.vtor.write(addr as u32);
    }

    asm::dsb();
    asm::isb();

    unsafe {
        interrupt::enable();
        asm::bootload(addr as *const u32);
    }
}
