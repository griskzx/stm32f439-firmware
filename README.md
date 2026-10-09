# STM32F439 固件

Cargo workspace 包含三个项目：

- `app`：Embassy 异步应用，当前仅初始化芯片并每秒输出日志。
- `boot`：基于 `stm32f4xx-hal` 的同步入口，尚未实现引导、升级或应用跳转。
- `xtask`：电脑端自动化任务入口，暂不引入额外依赖。

## 构建

需要支持 Rust 2024 edition 的工具链，并安装目标和 `flip-link` 链接器：

```sh
rustup target add thumbv7em-none-eabihf
cargo install flip-link --locked
```

app 和 boot 共用根目录的 `flip-link` 配置，为普通 SRAM 中的栈提供溢出保护；CCM 段仍独立放置，不受该保护覆盖。

在仓库根目录执行：

```sh
cargo build -p app --target thumbv7em-none-eabihf
cargo build -p boot --target thumbv7em-none-eabihf
cargo xtask help
cargo fmt --all -- --check
```

根目录不设置默认交叉编译目标，以保证 `xtask` 在电脑端运行。进入 `app` 或 `boot` 目录后，可直接执行 `cargo build`，子目录配置会选择 ARM 硬浮点目标。workspace 默认成员是 `xtask`。

## Zed / Rust Analyzer

`.zed/settings.json` 启用 build scripts、过程宏和保存检查，固件语义分析目标为 `thumbv7em-none-eabihf`。编辑器使用 Zed 管理的 Rust Analyzer；需要安装 `rust-src` 才能完整分析标准库源码：

```sh
rustup component add rust-src
```

`.zed/check.py` 使用 Python 标准库，可在 Linux 和 Windows 上运行。它分别检查 app/boot 的 ARM 目标（包含 boot 测试）和 xtask 的电脑端目标；电脑端目标由 `rustc -vV` 自动识别，不硬编码操作系统。构建脚本和保存检查共用此命令，输出 LSP 所需的 Cargo JSON 诊断，产物隔离到 `target/rust-analyzer`。

需要安装 Python 3.9 或更新版本，并确保 Zed 启动环境的 PATH 中可以执行 `python3`。若 Windows 安装仅提供 `python` 命令，将 `.zed/settings.json` 中两处 `python3` 改为 `python`。可在根目录手动验证：

```sh
python3 .zed/check.py
```

注意：按包分别编译检查不等于 Rust Analyzer 同时使用两个语义分析目标。当前语义分析以固件为主；若 xtask 后续大量使用平台相关 `cfg`，需另行配置电脑端分析环境。

## 烧录与日志

安装 probe-rs 并连接调试器后，在根目录执行：

```sh
cargo run -p app --target thumbv7em-none-eabihf
cargo run -p boot --target thumbv7em-none-eabihf
```

当前配置对应 STM32F439ZI，默认 RTT 日志级别为 `info`。

**目前两个固件均从 `0x08000000` 启动，独立使用完整 Flash，烧录其中一个会覆盖另一个，不能作为 boot + app 组合同时使用。** 后续确定分区后，需要同步调整 boot 的 `memory.x`、app 的链接布局以及应用跳转逻辑。

boot 和 app 均使用各自显式的 `memory.x`：2 MiB Flash、192 KiB 普通 SRAM，以及从 `0x10000000` 开始的 64 KiB CCM。普通数据和栈仍放在 SRAM。

两边 `main` 都演示了将 1 KiB 缓冲区放入 `.ccm`：先显式清零，再写入并读取数值，通过 RTT 输出地址和值。`.ccm` 是 `NOLOAD` 段，不会由启动代码自动清零；必须在读取前自行初始化。CCM 仅供 CPU 访问，不能用于 DMA 缓冲区。示例中的缓冲区仅由主入口独占，后续共享给任务或中断时需要额外的同步与所有权设计。

app 的异步时钟占用 TIM4；后续使用该定时器前需调整 time-driver。

已移除复制示例中的网络、USB、存储等直接依赖；HAL 自身仍可能引入相关传递依赖。按实际功能需求再添加直接依赖和 features。
