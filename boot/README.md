# Boot

基于 `stm32f4xx-hal` 的 STM32F439ZI 同步固件入口。目前仅获取外设并输出初始化日志，尚未实现 bootloader 功能。

在本目录执行 `cargo build` 或 `cargo run`，默认目标为 `thumbv7em-none-eabihf`。烧录需要 probe-rs。

当前 `memory.x` 是独立固件布局，并未为 app 预留分区。构建命令、烧录注意事项及后续分区说明见仓库根目录的 [README](../README.md)。
