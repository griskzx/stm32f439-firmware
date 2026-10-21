#![no_main] // 使用下面的 cortex_m_rt::entry 作为入口，而不是操作系统的 main。
#![no_std] // 裸机程序没有标准库，使用 core 和硬件驱动。

use boot as _; // 引入本项目库中的 RTT 日志、panic 和异常处理。
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};

use boot::{parse,usb};

// NUCLEO-F439ZI（MB1137）的 ST-LINK 提供 8 MHz MCO 时钟给 PH0/OSC_IN。
// 这是外部输入的时钟信号，因此使用 HSE bypass（旁路模式）。
const HSE_HZ: u32 = 8_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    // 获取芯片外设的唯一所有权；take() 只允许成功一次，避免重复控制同一外设。
    let dp = pac::Peripherals::take().unwrap();

    defmt::info!(
        "Boot: initializing HSE bypass from ST-LINK MCO ({} Hz)",
        HSE_HZ
    );

    // 配置 PLL 倍频与总线分频。168 MHz 不需要开启 OverDrive。
    // freeze 会实际设置硬件、配置 Flash 等待周期，并等待时钟稳定。
    let mut rcc = dp.RCC.freeze(
        Config::hse(HSE_HZ.Hz()) // 外部时钟输入为 8 MHz。
            .bypass_hse_oscillator() // 使用时钟输入，而不是驱动外部晶振。
            .sysclk(168.MHz()) // 系统时钟。
            .hclk(168.MHz()) // CPU / AHB 总线时钟。
            .pclk1(42.MHz()) // APB1 外设总线时钟。
            .pclk2(84.MHz()) // APB2 外设总线时钟。
            .require_pll48clk(), // 另外要求 PLL 为 USB 生成准确的 48 MHz。
    );

    // 读取硬件状态，确认系统时钟源已经切换到 PLL。
    while !rcc.cfgr().read().sws().is_pll() {}

    defmt::info!("Boot: clock source = HSE bypass -> PLL");
    defmt::info!(
        "Boot: SYSCLK={} Hz, HCLK={} Hz, PCLK1={} Hz, PCLK2={} Hz",
        rcc.clocks.sysclk().to_Hz(),
        rcc.clocks.hclk().to_Hz(),
        rcc.clocks.pclk1().to_Hz(),
        rcc.clocks.pclk2().to_Hz(),
    );

    // 把 USB 和对应 GPIO 的所有权交给 usb.rs。
    // init 会开启 OTG_FS 中断后返回；之后 USB 的 poll 和收发都在中断里完成，
    // 主循环只通过 rx / tx 队列读写数据，耗时操作不会影响 USB 连接。
    let mut serial = usb::init(
        dp.GPIOA,
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        &mut rcc,
    );

    let cli = parse::Cli;
    // USB CDC 是字节流，一次读到的数据不一定是一条完整命令，所以按行累计。
    let mut line: heapless::Vec<u8, 64> = heapless::Vec::new();

    loop {
        while let Some(byte) = serial.read_byte() {
            // 回显收到的字节；队列满时丢弃回显，不影响命令解析。
            serial.write(&[byte]);

            match byte {
                b'\n' => {
                    // 支持 \r\n 和 \n 两种行结束符。
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    cli.cmd_parse(&line);
                    line.clear();
                }
                // 行太长时 push 失败，直接丢弃这一字节。
                _ => {
                    let _ = line.push(byte);
                }
            }
        }
    }
}
