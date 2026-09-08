# joc-rtos-app-sdk

> jOS RTOS（joc-base 自研 RTOS）的 **Rust 应用层 SDK（中间工程）**：任何 Rust 应用
> 只需依赖本 SDK 并实现一个 `app_main()`，即可编译成独立 App 分区（轨 B 双分区），
> 跑在 joc-base 的 jOS RTOS 上——不依赖 mavlink-core / flyctrl / 任何缺失仓库。

## 为什么需要它

- joc-app-rust（飞控应用）目前**不完整**：依赖 `mavlink-core` 仓库（未克隆、GitHub 私有/不存在），
  无法独立构建。
- 本 SDK 复刻 joc-app-rust 已验证的 ABI 挂载链路（`g_app_slot` 服务表 + app.ld 分区头部 +
  系统自举），把它做成**可复用中间工程**：
  - SDK 负责一切样板：入口 `rust_app_start`（.data 自拷贝 + 版本校验 + 挂载自报）、
    ABI 绑定、任务创建、设备访问、日志、panic 处理；
  - 具体应用只写业务：`app_main()` 里拉起自己的 RTOS 任务。

## 目录结构

```
joc-rtos-app-sdk/
├── sdk/                  # rtos-app-sdk crate（rlib+staticlib）：ABI/任务/设备/日志/挂载自举
│   └── src/{lib,abi,rtos,device,log}.rs
├── apps/demo/            # demo-app（staticlib）：最小周期任务示例，零外设依赖
├── linker/app.ld         # 应用分区链接脚本（生成 app_header_t + XIP 代码 + APP_RAM .data/.bss）
├── abi/rtos_abi.h        # 契约头（从 joc-base 同步；build.rs 校验版本）
├── build_app.py          # 一键打包：cargo → libapp.a → app.elf → app.bin
└── Cargo.toml            # workspace
```

## 快速开始

```bash
# 前置：rustup(stable + thumbv7em-none-eabihf)、arm-none-eabi-gcc/objcopy
python build_app.py            # 产出 app.bin（demo 应用）
python build_app.py --check    # 仅校验工具链

# 烧录/模拟：app.bin 烧到 APP_FLASH (0x08060000)
#   真机:  joc-base 的 flash_app.bat
#   模拟:  mcu_simulater 的 load_app_partition()（x_jos_app 测试 / cfg-run）
```

## 如何写你自己的 Rust 应用（集成 SDK）

```toml
# 你的 Cargo.toml
[lib]
crate-type = ["staticlib"]
[dependencies]
rtos-app-sdk = { path = "../joc-rtos-app-sdk/sdk" }
```

```rust
#![no_std]
use rtos_app_sdk::info;
use rtos_app_sdk::rtos::{spawn, msleep, RTOS_PRIO_BH_MED};

#[link_section = ".rust_bss"]
static mut MY_STACK: [u8; 4096] = [0u8; 4096];

extern "C" fn my_task(_arg: *mut core::ffi::c_void) {
    info!("myapp", "task started");
    loop { info!("myapp", "alive ticks={}", rtos_app_sdk::rtos::tick_count()); msleep(50); }
}

#[no_mangle]
pub extern "C" fn app_main() -> i32 {
    unsafe { spawn("my_app", my_task, RTOS_PRIO_BH_MED, MY_STACK.as_mut_ptr(), MY_STACK.len()); }
    0
}
```

然后 `python build_app.py --app apps/my-app`（或把包名改为自己的）产出 app.bin。

## SDK 提供的能力（均经 g_app_slot 间接调用，App 独立链接安全）

| 模块 | 能力 |
|---|---|
| `abi` | `g_app_slot` 服务表镜像、`app_slot_t`/`rtos_sem_t`/`rtos_mq_t` 等 `#[repr(C)]` 结构、优先级/RT 类别常量、ABI 版本 |
| `rtos` | `spawn`/`spawn_rt`（创建任务）、`msleep`/`tick_count`/`cycle_now`、`Semaphore`/`Mutex` |
| `device` | `Device::get/read/write/ioctl`（uart0/usb0/...），不碰裸寄存器 |
| `log` | `info!/warn!/error!`（`core::fmt` 格式化 → uart0 控制台，前缀 `R/{L}/{tag}`） |
| 入口 | `rust_app_start`（.data 自拷贝 + 版本校验 + "RUST app mounted" 自报 + 调 `app_main`）+ panic→UDF |

## ABI 契约要点（与 joc-base 严格一致）

- **App 分区头部**（app.ld 生成，32B）：`magic=0x41504800("APH\0")` / `abi_version=1` /
  `entry=rust_app_start 绝对地址` / `app_size` / reserved。
- **APP_FLASH** = 0x08060000（sector 7），**APP_RAM** = 0x20004000（111KB，发布版），
  **g_app_slot** = 0x2001FC00（系统侧固定地址，App 只引用不定义）。
- 系统加载器：校验 magic/abi → 清零 APP_RAM → 建 `app_host` 任务调用 `entry`。
- App 侧 `.data` 自拷贝（LMA=Flash → VMA=RAM）；`.bss` 由系统清零。
- 契约变更必须同步 `abi/rtos_abi.h` 与 `sdk/src/abi.rs` 并 +`RTOS_ABI_VERSION`（build.rs 强制校验）。

## 已在 mcu_simulater 上验证

`mcu_simulater` 加载 `joc-base` 发布固件 + 本 SDK 的 `app.bin`（x_jos_app 测试）：
系统启动 → App 分区发现（entry=0x080607A4）→ app_host 任务 → `rust_app_start`
→ **"RUST app mounted"** → demo 任务创建 → **"task started"** → 全部通过。

> 已打通：任务进入周期睡眠后由 SysTick 周期唤醒（PendSV 高密度风暴已修复，
> 见 mcu_simulater CONTRIBUTING-run.md §二.4）。模拟器上 demo 任务每虚拟秒打一条
> `alive seq=`（x_jos_app / x_jos_hb 均已把心跳纳入回归断言）。
