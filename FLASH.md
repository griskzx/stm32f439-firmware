# STM32F439ZI Flash 内存结构与读写规则手册

> **目标芯片**：STM32F439ZI (Cortex-M4F, LQFP-144)  
> **板载开发板**：NUCLEO-F439ZI  
> **文档用途**：作为固件开发（Bootloader、App 固件跳转、OTA 升级、配置参数存储）的速查手册。

---

## 一、Flash 核心规格概览

| 参数项 | 规格说明 |
| :--- | :--- |
| **总容量** | **2048 KB (2 MB)** |
| **物理架构** | **Dual-Bank（双 Bank）架构**（默认模式，两个独立的 1 MB Bank） |
| **地址映射** | `0x0800_0000` ～ `0x081F_FFFF` |
| **扇区总数** | 共 **24 个扇区**（Sector 0 ～ Sector 23） |
| **擦除粒度** | **仅支持按扇区（Sector）或全片/全 Bank 擦除**，无单字节擦除 |
| **写入位宽** | 3.3V 供电下支持 **8 位（字节）、16 位（半字）、32 位（字）** 编程 |
| **RWW 特性** | **支持 Read-While-Write（读写同时进行）**：在 Bank 1 执行代码时，可对 Bank 2 进行擦写而不阻塞 CPU |
| **擦写寿命** | 典型值 10,000 次（全温范围），数据保持时间 10~20 年 |

---

## 二、Flash 物理扇区分布表（Dual-Bank 模式）

STM32F439ZI 拥有 2 个独立的 1MB Bank，每个 Bank 内由 **4 个 16KB、1 个 64KB、7 个 128KB** 扇区组成：

### 1. Bank 1（主存储区 1：共 1024 KB）

| 扇区编号 | 扇区大小 | 物理起始地址 | 物理结束地址 | 常见规划建议 |
| :---: | :---: | :---: | :---: | :--- |
| **Sector 0** | 16 KB | `0x0800_0000` | `0x0800_3FFF` | Bootloader 中断向量表与启动代码 |
| **Sector 1** | 16 KB | `0x0800_4000` | `0x0800_7FFF` | Bootloader 代码 |
| **Sector 2** | 16 KB | `0x0800_8000` | `0x0800_BFFF` | Bootloader 代码 / 启动参数配置（KV 存储） |
| **Sector 3** | 16 KB | `0x0800_C000` | `0x0800_FFFF` | 启动标志 / 备份配置区 |
| **Sector 4** | 64 KB | `0x0801_0000` | `0x0801_FFFF` | Bootloader 扩展 / 驱动代码 |
| **Sector 5** | 128 KB | `0x0802_0000` | `0x0803_FFFF` | **App 应用程序起始位置（偏移 128KB）** |
| **Sector 6** | 128 KB | `0x0804_0000` | `0x0805_FFFF` | App 应用程序代码 |
| **Sector 7** | 128 KB | `0x0806_0000` | `0x0807_FFFF` | App 应用程序代码 |
| **Sector 8** | 128 KB | `0x0808_0000` | `0x0809_FFFF` | App 应用程序代码 |
| **Sector 9** | 128 KB | `0x080A_0000` | `0x080B_FFFF` | App 应用程序代码 |
| **Sector 10** | 128 KB | `0x080C_0000` | `0x080D_FFFF` | App 应用程序代码 |
| **Sector 11** | 128 KB | `0x080E_0000` | `0x080F_FFFF` | App 应用程序代码 / 用户大文件存储 |

> **提示**：Sector 0 ~ 4 的总容量为 `16 + 16 + 16 + 16 + 64 = 128 KB`，因此 Sector 5 的起始地址正好为 `0x0802_0000`，常作为 App 分区的起点。

### 2. Bank 2（主存储区 2：共 1024 KB）

| 扇区编号 | 扇区大小 | 物理起始地址 | 物理结束地址 | 常见规划建议 |
| :---: | :---: | :---: | :---: | :--- |
| **Sector 12** | 16 KB | `0x0810_0000` | `0x0810_3FFF` | OTA 固件下载暂存区 / App 槽位 B 起点 |
| **Sector 13** | 16 KB | `0x0810_4000` | `0x0810_7FFF` | 暂存区 / 状态标志 |
| **Sector 14** | 16 KB | `0x0810_8000` | `0x0810_BFFF` | 校准参数 / 数据备份 |
| **Sector 15** | 16 KB | `0x0810_C000` | `0x0810_FFFF` | 离线日志 / 历史记录 |
| **Sector 16** | 64 KB | `0x0811_0000` | `0x0811_FFFF` | OTA 下载区 |
| **Sector 17** | 128 KB | `0x0812_0000` | `0x0813_FFFF` | OTA 下载区 |
| **Sector 18** | 128 KB | `0x0814_0000` | `0x0815_FFFF` | OTA 下载区 |
| **Sector 19** | 128 KB | `0x0816_0000` | `0x0817_FFFF` | OTA 下载区 |
| **Sector 20** | 128 KB | `0x0818_0000` | `0x0819_FFFF` | OTA 下载区 |
| **Sector 21** | 128 KB | `0x081A_0000` | `0x081B_FFFF` | OTA 下载区 |
| **Sector 22** | 128 KB | `0x081C_0000` | `0x081D_FFFF` | OTA 下载区 |
| **Sector 23** | 128 KB | `0x081E_0000` | `0x081F_FFFF` | OTA 下载区 |

### 3. 系统附属区域

- **System Memory（系统存储区）**：`0x1FFF_0000 ～ 0x1FFF_77DF`（30 KB），ST 出厂内置系统 Bootloader（支持串口/USB DFU 烧录）。
- **OTP（One-Time Programmable 一次性编程区）**：`0x1FFF_7800 ～ 0x1FFF_7A0F`（512 字节用户数据 + 16 字节锁定字节），只能写一次，无法擦除。
- **Option Bytes（选项字节）**：`0x1FFF_C000 ～ 0x1FFF_C00F`（Bank 1）及 `0x1FFF_C020 ～ 0x1FFF_C02F`（Bank 2），用于配置读写保护级别、看门狗硬件模式、Dual-Bank 切换等。

---

## 三、Flash 读写操作核心黄金法则

### 规则 1：读取极其简单（直接内存映射）
- Flash 挂载在 Cortex-M 的统一编址空间内，**读取 Flash 和读取 RAM 数组没有任何本质区别**。
- 可以直接通过裸指针、切片（Slice）引用：
  ```rust
  let ptr = 0x0802_0000 as *const u8;
  let slice = unsafe { core::slice::from_raw_parts(ptr, 64) };
  ```
- **注意时钟等待周期（Latency）**：STM32F439 在 180 MHz 时钟下，Flash 访问需要设置 **5 个等待周期（WS）**，通常在系统初始化配置 RCC 时由 HAL 库自动配好。

---

### 规则 2：物理特性——只能从 1 改写为 0
- 擦除后的 Flash 存储单元每个 bit 都是 **`1`**（整字节表现为 **`0xFF`**）。
- 写入（编程）只能执行 **`1 -> 0`** 的跳变。
- **如果某 bit 已经是 `0`，无法通过直接写入变回 `1`**。试图向已有非 `0xFF` 的位置写入新数据会导致数据错误或触发 Flash 控制器编程错误。若要写回 `1`，必须进行**扇区擦除**。

---

### 规则 3：必须按“扇区（Sector）”擦除，不可按字节擦除
- Flash 的擦除单位只有 **整个扇区**（16KB、64KB 或 128KB），或者全片擦除。
- **如果只想修改扇区里的 1 个字节**：
  1. 必须先把整个扇区读入 RAM 缓冲区；
  2. 在 RAM 里修改该字节；
  3. 执行扇区擦除（整个扇区恢复全 `0xFF`）；
  4. 将 RAM 数据重新写入该扇区。
- **参数存储推荐实践**：若保存小型参数，建议采用 **追加写入（Append-Only / Log-structured）** 模式，写满一个扇区后再统一垃圾回收（GC），避免频繁擦除。

---

### 规则 4：写入前必须解锁，写完必须重新上锁
Flash 控制器寄存器（`FLASH_CR`）默认带有硬件写保护，防止程序意外跑飞篡改代码：
- **解锁操作**：按固定顺序向 `FLASH_KEYR` 写入两个特殊密钥：
  1. `KEY1 = 0x4567_0123`
  2. `KEY2 = 0xCDEF_89AB`
- **上锁操作**：完成擦写后，将 `FLASH_CR` 的 `LOCK` 位置 1。
- 在 Rust `stm32f4xx-hal` 中，推荐使用 **RAII 机制**（调用 `.unlocked()`，离开作用域自动 `drop` 上锁）。

---

### 规则 5：写入对齐与供电电压（PSIZE）
在 Nucleo 开发板（3.3V 正常供电，$V_{DD} \ge 2.7\text{V}$）下：
- 硬件支持 **x8（字节）、x16（半字）、x32（32位字）** 写入。
- 编程 32 位字时，目标地址**必须 4 字节对齐**（地址后两位为 `0b00`），否则会产生对齐错误（PGAERR）。

---

### 规则 6：擦除耗时极长，注意看门狗与中断
- **128 KB 大扇区擦除时间约为 1 ～ 2 秒**；16 KB 扇区约需数百毫秒。
- 在擦除或写入时，Flash 控制器会将状态寄存器的 **`BSY`（Busy）位**置 1。
- **重大陷阱**：
  - 如果正在擦写 Bank 1，且中断向量表或中断服务函数（ISR）也位于 Bank 1，**CPU 会被硬性挂起（Stall）**，直到 Flash 擦写完成。如果擦写超时或有独立看门狗（IWDG）且未及时喂狗，系统会被复位。
  - **规避建议**：关键固件擦写前关闭全局中断，或将中断向量表重定位到 SRAM。

---

### 规则 7：Dual-Bank 的 RWW（Read-While-Write）神技
- STM32F439ZI 的双 Bank 是物理独立的：
  - **当程序在 Bank 1 执行时，可以对 Bank 2 进行擦除或写入，CPU 不会发生流水线挂起（Stall）**。
  - 这对于 **后台 OTA 下载** 极其有利（边运行 Bank 1 的主程序，边接收数据写入 Bank 2，互不干扰）。

---

## 四、Rust 代码实战示例（基于 `stm32f4xx-hal`）

参考项目中的 [`boot/src/bin/flash_demo.rs`](file:///C:/Users/HP/RustroverProjects/stm32f439-firmware/boot/src/bin/flash_demo.rs)：

```rust
use stm32f4xx_hal::{flash::FlashExt, pac::Peripherals};

// 1. 获取外设并解锁 Flash
let mut dp = Peripherals::take().unwrap();
let mut unlocked_flash = dp.FLASH.unlocked();

// 2. 扇区擦除（以 Sector 5 为例：起始物理地址 0x0802_0000）
const SECTOR_NUM: u8 = 5;
unlocked_flash.erase(SECTOR_NUM).expect("扇区擦除失败");

// 3. 数据写入
// 注意：TARGET_OFFSET 是相对 Flash 基地址 0x0800_0000 的字节偏移量
// Sector 5 的偏移量为 128 * 1024 = 0x20000
const TARGET_OFFSET: usize = 128 * 1024;
let data_to_write: &[u8] = b"Firmware Configuration Data V1.0";

unlocked_flash
    .program(TARGET_OFFSET, data_to_write.iter())
    .expect("Flash 写入失败");

// 4. 显式释放以重新上锁
drop(unlocked_flash);

// 5. 读取数据（零拷贝，直接切片读取）
let target_addr = (0x0800_0000 + TARGET_OFFSET) as *const u8;
let read_data = unsafe { core::slice::from_raw_parts(target_addr, data_to_write.len()) };

if read_data == data_to_write {
    defmt::info!("Flash 写入与校验成功！");
}
```

---

## 五、项目常见内存划分推荐（Bootloader + App）

```text
0x0800_0000 +------------------------------------------+  0 KB
            | Sector 0 ~ 4 (128 KB)                    |
            | -> Bootloader 分区 (boot crate)           |
0x0802_0000 +------------------------------------------+  128 KB
            | Sector 5 ~ 11 (896 KB)                   |
            | -> App 应用程序主分区 (app crate)          |
0x0810_0000 +------------------------------------------+  1024 KB (1 MB)
            | Sector 12 ~ 23 (1024 KB, Bank 2)         |
            | -> OTA 升级固件暂存区 / 外部数据持久化区    |
0x0820_0000 +------------------------------------------+  2048 KB (2 MB)
```
