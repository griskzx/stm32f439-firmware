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

App 使用 Cargo feature 选择 A/B 分区，不需要手动设置环境变量。默认构建 App A：

```bash
cargo app-a
```

构建 App B：

```bash
cargo app-b
```

也可以使用完整命令：

```bash
cargo build -p app --release
cargo build -p app --release --no-default-features --features slot-b
```

也可以继续使用根配置中的缩写：

```bash
cargo rb hello
cargo rb app -p app
```

## Flash 分区

STM32F439 的 2 MB Flash 分区如下：

| 区域 | 起始地址 | 大小 | 扇区 |
|---|---:|---:|---:|
| Bootloader | `0x08000000` | 128 KB | 0..4 |
| App A | `0x08020000` | 896 KB | 5..11 |
| App B | `0x08100000` | 896 KB | 12..22 |
| Metadata | `0x081E0000` | 128 KB | 23 |

App 的链接脚本位于 `app/memory-a.x` 和 `app/memory-b.x`。`app/build.rs`
会根据 Cargo feature 将选中的脚本复制为链接器使用的 `memory.x`：

```text
slot-a -> 0x08020000
slot-b -> 0x08100000
```

Bootloader 仍需在运行时校验镜像、选择活动槽位、设置 VTOR，并跳转到对应的
Reset Handler；链接脚本只负责生成正确的固件地址布局。

原项目模板的详细说明保留在
[`bootloader/README.md`](bootloader/README.md)。
