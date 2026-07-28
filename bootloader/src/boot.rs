use cortex_m::{
    asm,
    interrupt,
    peripheral::SCB,
};
const APP_A:u32 = 0x08020000;
const APP_B:u32 = 0x08100000;

#[allow(unreachable_code)]
pub unsafe fn jump_to_app(app_addr:u32)->!{
    // 关闭中断
    interrupt::disable();

    // 设置向量表
    unsafe {
        (*SCB::PTR).vtor.write(app_addr);
        asm::bootload(app_addr as *const u32);
    }
    loop {}
}