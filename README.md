# STM32F439 Bootloader Workspace

这个仓库现在由 Cargo Workspace 统一管理两个独立的 Rust 嵌入式项目：

```text
.
├── Cargo.toml          # Workspace 与共享依赖
├── .cargo/config.toml  # STM32F439 编译目标、下载器与日志配置
├── bootloader/         # 原有 bootloader 项目
└── app/                # 新应用模板
```

## 常用命令

检查整个工作区：

```bash
cargo check --workspace
```

构建或运行 Bootloader 中的某个二进制：

```bash
cargo build -p bootloader --bin hello
cargo run -p bootloader --bin hello
```

构建或运行应用：

```bash
cargo build -p app
cargo run -p app
```

也可以继续使用根配置中的缩写：

```bash
cargo rb hello
cargo rb app -p app
```

## Flash 分区

`bootloader/memory.x` 与 `app/memory.x` 已经彼此独立。当前两者都沿用完整的
STM32F439 Flash 地址范围，便于分别构建和下载。真正让 Bootloader 跳转到 App
之前，需要根据 Bootloader 的预留大小修改两份 `memory.x` 中的 `FLASH` 区域，
并在 Bootloader 中实现向量表重定位与跳转逻辑。

原项目模板的详细说明保留在
[`bootloader/README.md`](bootloader/README.md)。
