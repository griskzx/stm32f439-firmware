# USB CDC Boot CLI demo

在 NUCLEO-F439ZI（MB1137）上通过 MCU 的 USB OTG FS 提供简易串口 CLI。
Boot 固定使用 `0x08000000` 起的 128 KiB 分区，启动后一直等待命令。

## 连接和运行

1. 保留板上的默认 ST-LINK MCO 配置；时钟为 8 MHz HSE bypass → 168 MHz SYSCLK，PLL 同时提供 48 MHz USB 时钟。需要保持 ST-LINK 供电，才能获得 MCO。
2. 通过 ST-LINK USB（CN1）供电、烧录，再用一根**数据线**将用户 USB Micro-AB（CN13）连接到电脑。USB D− 为 PA11，D+ 为 PA12。板级连接参见 [ST UM1974](https://www.st.com/resource/en/user_manual/um1974-stm32h743-nucleo144-board-stmicroelectronics.pdf) 的 USB 章节。
3. 在本目录执行：

   ```powershell
   cargo build --locked -j 1
   cargo run --locked -j 1
   ```

   `cargo run` 使用已有的 probe-rs runner，通过 ST-LINK 烧录并启动 Boot。
4. 打开新出现的 `STM32F439 Boot CLI` COM 口，**启用 DTR，关闭本地回显和硬件流控**。可选 115200 / 8N1；CDC 的波特率设置不影响实际 USB 传输速度。
   此 COM 口属于 CN13 上的 MCU USB，ST-LINK 自带的虚拟串口用于 UART，不能访问本 CLI。
5. 看到 `boot> ` 后输入 `help`。支持 CR、LF、CRLF、Backspace、Delete、Ctrl-C，以及一次粘贴多条命令。

示例：

```text
boot> info
boot 0.1.0 / STM32F439ZI
UID=...
SYSCLK=168000000 HCLK=168000000 Hz
PCLK1=42000000 PCLK2=84000000 USB=48000000 Hz
boot> slots
A @ 08020000: MSP=FFFFFFFF RESET=FFFFFFFF invalid
B @ 08120000: MSP=FFFFFFFF RESET=FFFFFFFF invalid
Vector check only; CRC/signature/boot state not implemented.
boot>
```

上面 `slots` 示例假定两槽为空，实际结果取决于 Flash 内容。

## 命令

| 命令 | 功能 |
|---|---|
| `help` / `?` | 列出命令 |
| `info` | Boot 版本、芯片 UID、各总线和 USB 时钟 |
| `layout` | 显示 [Flash 分区规划](../FLASH_LAYOUT.md) |
| `slots` | 查看 A/B 初始 MSP 与复位向量，检查 RAM 范围、8 字节栈对齐、Thumb 位与槽位范围 |
| `reboot` | 等待最多 250 ms 发送回执，然后系统复位，重新进入 CLI；端口可能需要重新打开 |

命令区分大小写，目前均不接受参数。单行最多 128 字节，超长或含不支持控制字节的整行会丢弃，避免执行截断后的命令。发送忙时保留未发送数据并继续 USB 轮询；重新打开端口会清除未完成输入。

此 demo 实现通信和命令入口。`slots` 只做向量合理性检查，**不表示固件完整或可安全启动**。尚未实现 Flash 擦写、固件下载、CRC/签名、App 跳转、启动状态保存和回滚。
当前 `app/memory.x` 仍为从 `0x08000000` 开始的独立 App；直接烧录它会覆盖 Boot。后续实现启动命令前，需要先将 App 按 A/B 地址链接并完成镜像校验和外设交接。

USB VID/PID 为 HAL 示例中的开发用 `16C0:27DD`；USB 序列号来自芯片 UID。轮询模式便于调试，当前会持续占用 CPU。实现参考 [stm32f4xx-hal 0.23.0 USB 示例](https://docs.rs/crate/stm32f4xx-hal/0.23.0/source/examples/usb-serial-poll.rs)。

## 代码与验证

- `src/main.rs`：时钟与入口。
- `src/usb_cli.rs`：USB 描述符、收发、DTR 会话和延迟复位。
- `src/cli.rs`：不依赖硬件的行编辑与命令处理；后续在 `dispatch` 和 `Action` 中扩展命令。

本机解析测试不需要连接板子，在本目录运行：

```powershell
New-Item -ItemType Directory -Force target | Out-Null
rustc --edition=2024 --test src/cli.rs -o target/cli-tests.exe
./target/cli-tests.exe
cargo clippy --bin boot --locked -j 1 -- -D warnings
```

上板检查：打开端口后依次发送 `help`、`info`、`layout`、`slots`；粘贴 `help\r\ninfo\r\n` 对应的两行文本，确认各执行一次；测试退格、Ctrl-C、超长行和重新连接；最后发送 `reboot`，确认设备重新枚举且能再次进入 CLI。

本次验证：调试构建、Clippy（禁止 warnings）和 6 项主机测试通过；ELF 中 Flash 内容约 45.6 KiB，符合 128 KiB Boot 分区限制。Release 构建在当前 Windows 环境中因内存/页面文件不足失败（单线程仍失败）；尚未进行上板验证。
