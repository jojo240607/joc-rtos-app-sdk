//! rtos-app-sdk：jOS RTOS 应用层 SDK（中间工程）。
//!
//! 任何 Rust 应用只需：
//!   1) 在 Cargo.toml 依赖 `rtos-app-sdk`（path）；
//!   2) 实现 `#[no_mangle] pub extern "C" fn app_main() -> i32`（创建你的业务任务）；
//!   3) 用仓库 `build_app.py`（app.ld）打包成 app.bin 烧到 APP_FLASH（0x08060000）。
//! SDK 负责：入口 `rust_app_start`（.data 自拷贝 + ABI 版本校验 + 挂载自报 + 调 app_main）、
//! ABI 绑定（g_app_slot）、任务/设备/日志封装、panic 处理。
//!
//! 挂载链路：RTOS 侧 app_slot_load_app() 校验分区头部后，在独立 app_host 任务里调用
//! `rust_app_start`（轨 B 应用分区）；本层经 g_app_slot 服务表间接调用内核，创建业务任务。

#![no_std]
#![allow(static_mut_refs)]

pub mod abi;
pub mod device;
pub mod ioctl;
pub mod irq;
pub mod log;
pub mod rtos;

// 链接脚本（app.ld）提供的 .data 段边界符号（LMA=Flash 初值地址，VMA=RAM 运行地址）。
extern "C" {
    static _appdata_lma: u8;
    static mut _sappdata: u8;
    static mut _eappdata: u8;
}

// 应用入口钩子：由具体应用实现（`#[no_mangle] pub extern "C" fn app_main() -> i32`）。
// 返回 0=OK；SDK 的 rust_app_start 把该返回值透传给 RTOS（app_host 打印 rc）。
extern "C" {
    fn app_main() -> i32;
}

/* ===========================================================================
 * 统一挂载点：由 RTOS app_host 任务调用（应用分区 header.entry -> rust_app_start）。
 * 做四件事：.data 自拷贝 / ABI 版本校验 / 挂载自报 / 调用应用提供的 app_main()。
 * ========================================================================= */
#[no_mangle]
pub extern "C" fn rust_app_start() -> i32 {
    unsafe {
        // 1) 初始化 .data：把 Flash LMA 处的初值拷贝到 RAM VMA。
        //    系统加载器仅清零 App .bss（APP_RAM），不拷贝 .data；App 独立镜像必须自拷贝。
        let src = _appdata_lma as *const u8;
        let dst = _sappdata as *mut u8;
        let n = (_eappdata as usize) - (_sappdata as usize);
        for i in 0..n {
            *dst.add(i) = *src.add(i);
        }

        // 2) 双重防御：版本不符直接拒绝挂载（build.rs 已做链接期校验）。
        let slot = &*core::ptr::addr_of!(abi::g_app_slot);
        if slot.magic != abi::APP_SLOT_MAGIC || slot.version != abi::RTOS_ABI_VERSION {
            log::raw(
                log::LEVEL_ERROR,
                "ABI mismatch: g_app_slot not initialized / version mismatch, refuse to mount",
            );
            return -1;
        }

        // 3) 挂载自报（x_jos_app 等验收脚本依据此标志确认 App 分区挂载成功）。
        info!("app_slot", "RUST app mounted (rust_app_start)");
    }

    // 4) 调用应用实现的 app_main()，由应用拉起业务任务。
    unsafe { app_main() }
}

/* ===========================================================================
 * panic 处理：abort -> UDF，触发内核 fault handler（可联动 WDT / 重启）。
 * ========================================================================= */
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // 触发 UDF 未定义指令异常，由 RTOS fault handler 捕获/恢复。
    unsafe { core::arch::asm!("udf #0", options(noreturn)) };
}
