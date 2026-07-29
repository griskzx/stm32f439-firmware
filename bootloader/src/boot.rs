use cortex_m::asm;
use cortex_m::interrupt;
use cortex_m::peripheral::SCB;
pub const APP_A: u32 = 0x08020000;
pub const APP_B: u32 = 0x08100000;

///
pub struct BootHeader {
    version: [u8; 4],
    active_slot: u32,
    crc: u32,
}

// #[allow(unreachable_code)]
pub unsafe fn jump_to_app(app_addr: u32) -> ! {
    unsafe {
        clear_device();

        (*SCB::PTR).vtor.write(app_addr);

        asm::dsb();
        asm::isb();

        // 恢复中断
        interrupt::enable();
        asm::bootload(app_addr as *const u32);
    }
    // loop {}
}

/// 清理:清理中断
pub unsafe fn clear_device() {
    unsafe {
        // 关闭中断
        interrupt::disable();

        // 关闭并清零 SysTick
        // CTRL
        core::ptr::write_volatile(0xE000_E010u32 as *mut u32, 0);
        // LOAD
        core::ptr::write_volatile(0xE000_E014u32 as *mut u32, 0);
        // VAL
        core::ptr::write_volatile(0xE000_E018u32 as *mut u32, 0);

        for i in 0..3 {
            let icer = (0xE000_E180u32 + i * 4) as *mut u32;
            core::ptr::write_volatile(icer, 0xFFFF_FFFF);
            let icpr = (0xE000_E280u32 + i * 4) as *mut u32;
            core::ptr::write_volatile(icpr, 0xFFFF_FFFF);
        }

        // 清除 PendSV 和 SysTick 的 pending 状态
        (*SCB::PTR).icsr.write((1 << 27) | (1 << 25));
    }
}
