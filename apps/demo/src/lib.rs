//! demo-app：jOS RTOS 最小示例应用（验证「系统自举 → rust_app_start → SDK → 任务拉起」链路）。
//!
//! 任务：一个周期任务，每 50ms 醒来打一条 alive 日志（依赖 SysTick 唤醒 + 调度）。
//! 不碰任何外设、不依赖 mavlink-core / flyctrl —— 纯 SDK 依赖，任何环境可构建。

#![no_std]
#![allow(static_mut_refs)]

use core::ffi::c_void;

use rtos_app_sdk::info;
use rtos_app_sdk::rtos::{msleep, spawn, tick_count, RTOS_PRIO_BH_MED};

/// demo 任务独立栈（放 App RAM，4KB）。
#[link_section = ".rust_bss"]
static mut DEMO_STACK: [u8; 4096] = [0u8; 4096];

/// 任务入口：先打「task started」自报（验收脚本据此确认任务拉起），再周期打 alive。
extern "C" fn demo_task_entry(_arg: *mut c_void) {
    let mut n: u32 = 0;
    info!("demo", "task started (prio={})", RTOS_PRIO_BH_MED);
    loop {
        n = n.wrapping_add(1);
        // 每 20 拍打一条 alive（含 tick_count，证明 RTOS tick 在推进）。
        if n % 20 == 0 {
            info!("demo", "alive seq={} ticks={}", n, tick_count());
        }
        msleep(50);
    }
}

/// 应用入口（SDK 的 rust_app_start 调用）：拉起业务任务后返回 0。
#[no_mangle]
pub extern "C" fn app_main() -> i32 {
    unsafe {
        spawn(
            "demo_app",
            demo_task_entry,
            RTOS_PRIO_BH_MED, // 中优先，不抢硬实时
            DEMO_STACK.as_mut_ptr(),
            DEMO_STACK.len(),
        );
    }
    info!("demo", "demo task spawned (link verified)");
    0
}
