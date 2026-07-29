use cortex_m::interrupt;
use cortex_m::peripheral::SCB;
use cortex_m::asm;
const APP_A:u32 = 0x08020000;
const APP_B:u32 = 0x08100000;

#[allow(unreachable_code)]
pub unsafe fn jump_to_app(app_addr:u32)->!{
    clear_device();

    (*SCB::PTR).vtor.write(app_addr);

    asm::dsb();
    asm::isb();

    // 恢复中断
    interrupt::enable();
    asm::bootload(app_addr as *const u32);
    // loop {}
}

/// 清理:清理中断
 pub unsafe fn clear_device(){
    // 关闭中断
    interrupt::disable();

    // 关闭并清零 SysTick
    // CTRL
    core::ptr::write_volatile(0xE000_E010 as *mut u32,0);
    // LOAD
    core::ptr::write_volatile(0xE000_E014 as *mut u32,0);
    // VAL
    core::ptr::write_volatile(0xE000_E018 as *mut u32,0);

    for i in 0..3 {
        let icer =  (0xE000_E180+ i * 4) as *mut u32;
        core::ptr::write_volatile(icer, 0xFFFF_FFFF);
        let icpr = (0xE000_E280 + i * 4) as *mut u32;
        core::ptr::write_volatile(icpr, 0xFFFF_FFFF);
    }

    //
    (*SCB::PTR).icsr.write((1<<27) | (1<<25));
}