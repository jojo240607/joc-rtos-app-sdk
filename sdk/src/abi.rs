//! 手写镜像 tools/abi/rtos_abi.h + src/app_slot/app_slot.h（joc-base 系统侧）。
//! 字段顺序、调用约定须与 C 侧严格一致；C 侧字段变更须同步本文件并 +RTOS_ABI_VERSION。
//! App 只经 `g_app_slot` 函数指针表间接调用内核服务，绝不直接链接裸 RTOS 符号
//! （应用分区独立链接时没有 RTOS 实现，直接 extern 调用会链接失败）。

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_void};

/* ---- 优先级常量（来自 rtos.h / rtos_config.h 的公开档位） ---- */
pub const RTOS_PRIO_BH_HIGH: u8 = 4; /* 硬实时任务上限：prio <= 此值才允许 rt_class=RTOS_RT_HARD */
pub const RTOS_PRIO_BH_MED: u8 = 6;
pub const RTOS_PRIO_MAIN: u8 = 12;
pub const RTOS_PRIO_BLINK: u8 = 14;
pub const RTOS_PRIO_BIST: u8 = 24;
pub const RTOS_PRIO_IDLE: u8 = 31;

/* ---- 硬实时类别（rtos_task_attr_t.rt_class） ---- */
pub const RTOS_RT_NONE: u8 = 0;
pub const RTOS_RT_HARD: u8 = 1;
pub const RTOS_RT_SOFT: u8 = 2;

/* ===========================================================================
 * 任务管理
 * ========================================================================= */
pub type rtos_task_entry_t = extern "C" fn(*mut c_void);

#[repr(C)]
pub struct rtos_task_attr_t {
    pub rt_class: u8,
    pub deadline_ticks: u32,
    pub wcet_ticks: u32,
}

/* ===========================================================================
 * IPC 原语（精简声明：隐藏 task_t* 等内核内部类型，用 void* 代替）
 * ========================================================================= */
#[repr(C)]
pub struct rtos_sem_t {
    pub count: u32,
    pub limit: u32,
    pub waitq: *mut c_void,
}

#[repr(C)]
pub struct rtos_mutex_t {
    pub owner: *mut c_void, // task_t* opaque
    pub ceil_prio: u8,
    pub recursive: u8,
    pub rec_count: u8,
    pub waitq: *mut c_void,
}

#[repr(C)]
pub struct rtos_mq_t {
    pub buf: *mut u8,
    pub item_size: usize,
    pub cap: usize,
    pub count: usize,
    pub head: usize,
    pub recv_waitq: *mut c_void,
    pub send_waitq: *mut c_void,
}

#[repr(C)]
pub struct rtos_work_t {
    pub next: *mut rtos_work_t,
    pub fn_: Option<extern "C" fn(*mut c_void)>,
    pub arg: *mut c_void,
    /* ★design.md P2-2：per-item 预算/截止期（cycles，0=不限/无）*/
    pub budget_cycles: u32,
    pub deadline_cycles: u32,
    /* ★design.md §8：连续超时 → 降级 */
    pub miss_count: u32,
    pub degraded: u8,
    /* ★design.md §5：周期由 WorkItem 声明，队列自带调度器按 EDF 派发 */
    pub period_cycles: u32,
    pub next_run_cycles: u32,
    pub queued: u8,
    pub pnext: *mut rtos_work_t,
}

pub type rtos_timer_cb_t = extern "C" fn(*mut rtos_timer_t, *mut c_void);

#[repr(C)]
pub struct rtos_timer_t {
    pub name: *const c_char,
    pub cb: Option<rtos_timer_cb_t>,
    pub arg: *mut c_void,
    pub mode: i32,           /* rtos_timer_mode_t: 0=oneshot 1=periodic */
    pub period_ticks: u32,
    pub expire: u32,
    pub active: u8,
    pub pending: u8,
    pub next: *mut rtos_timer_t,
}

#[repr(C)]
pub struct rtos_event_t {
    pub flags: u32,
    pub waitq: *mut c_void,
}

/* ===========================================================================
 * app_slot_t 镜像（方案 Y 轻量版）：RTOS 暴露给 App 的「函数指针表 + 中断回调注册位」
 * 契约。g_app_slot 由系统在固定链接地址（APP_SLOT_RAM，发布版 0x2001FC00）定义并填充，
 * App 经 extern 引用，不可自行定义。
 * =========================================================================== */
pub const APP_SLOT_MAGIC: u32 = 0x4150_5053; // "APPS"
pub const APP_SLOT_VERSION: u32 = 2;
pub const APP_IRQ_REG_MAX: usize = 8;

/* 中断类别（镜像 irq.h irq_class_t） */
pub const IRQ_CLASS_NORMAL: u8 = 0;
pub const IRQ_CLASS_KERNEL: u8 = 1;
pub const IRQ_CLASS_ZERO_LATENCY: u8 = 2;

#[repr(C)]
pub struct app_irq_reg_t {
    pub used: u8,
    pub irq_id: u8,
    pub prio_class: u8, // 0=NORMAL,1=KERNEL,2=ZERO_LATENCY
    pub rt_class: u8,   // 0=普通,1=硬实时
    pub isr_cb: Option<extern "C" fn(*mut c_void)>,
    pub ctx: *mut c_void,
}

/* 设备 vtable 镜像（device 指针 opaque） */
#[repr(C)]
pub struct deviceVtable {
    pub open: Option<extern "C" fn(*mut c_void) -> i32>,
    pub close: Option<extern "C" fn(*mut c_void) -> i32>,
    pub read: Option<extern "C" fn(*mut c_void, *mut c_void, usize) -> i32>,
    pub write: Option<extern "C" fn(*mut c_void, *const c_void, usize) -> i32>,
    pub ioctl: Option<extern "C" fn(*mut c_void, i32, *mut c_void) -> i32>,
    pub irq_id: Option<extern "C" fn(*mut c_void) -> i32>,
}

#[repr(C)]
pub struct device_t {
    pub vtable: *const deviceVtable,
    pub type_: u32,
    pub name: *const c_char,
    pub class: u32,
}

/* ABI 版本：与 C 侧 tools/abi/rtos_abi.h 的 RTOS_ABI_VERSION 对齐（build.rs 校验）。 */
pub const RTOS_ABI_VERSION: u32 = 12;

pub type app_slot_irq_attach_t = extern "C" fn(*const app_irq_reg_t) -> i32;

#[repr(C)]
pub struct app_slot_t {
    pub magic: u32,
    pub version: u32,
    pub reserved: u32,

    /* 内核服务 */
    pub task_create: Option<
        extern "C" fn(
            *const c_char,
            rtos_task_entry_t,
            *mut c_void,
            u8,
            *mut c_void,
            usize,
        ),
    >,
    pub task_create_rt: Option<
        extern "C" fn(
            *const c_char,
            rtos_task_entry_t,
            *mut c_void,
            u8,
            *mut c_void,
            usize,
            u8,
            *const rtos_task_attr_t,
        ),
    >,
    pub msleep: Option<extern "C" fn(u32)>,
    pub tick_count: Option<extern "C" fn() -> u32>,
    pub cycle_now: Option<extern "C" fn() -> u32>,
    pub delay_until: Option<extern "C" fn(*mut u32, u32)>,

    /* IPC 服务 */
    pub sem_init: Option<extern "C" fn(*mut rtos_sem_t, u32, u32)>,
    pub sem_wait: Option<extern "C" fn(*mut rtos_sem_t) -> i32>,
    pub sem_trywait: Option<extern "C" fn(*mut rtos_sem_t) -> i32>,
    pub sem_give: Option<extern "C" fn(*mut rtos_sem_t)>,

    /* 设备服务 */
    pub dev_get: Option<extern "C" fn(*const c_char) -> *mut device_t>,
    pub dev_open: Option<extern "C" fn(*mut device_t) -> i32>,
    pub dev_read: Option<extern "C" fn(*mut device_t, *mut c_void, usize) -> i32>,
    pub dev_write: Option<extern "C" fn(*mut device_t, *const c_void, usize) -> i32>,
    pub dev_ioctl: Option<extern "C" fn(*mut device_t, i32, *mut c_void) -> i32>,
    pub dev_close: Option<extern "C" fn(*mut device_t) -> i32>,

    /* 中断回调注册位 */
    pub irq_reg: [app_irq_reg_t; APP_IRQ_REG_MAX],

    /* 系统注册入口 */
    pub irq_attach: Option<app_slot_irq_attach_t>,
    pub irq_enable: Option<extern "C" fn(u8) -> i32>,
    pub irq_disable: Option<extern "C" fn(u8) -> i32>,

    /* 生命周期 */
    pub app_start: Option<extern "C" fn() -> i32>,
    pub app_stop: Option<extern "C" fn()>,

    /* ★design.md P0-2：RT 违约计数（[deadline_violation, wcet_violation, sched_invalid]）
     * —— 让硬/软实时任务的 deadline/wcet 违约对 App 可见（可观测性）。 */
    pub rt_violation: Option<extern "C" fn(*mut u32)>,

    /* ★design.md P2-1：共享工作队列提交（`bh.h` 的 `rtos_work_t`）。
     * `w` 须由 App 静态分配；提交后共享 worker 执行 `w->fn(w->arg)`。 */
    pub work_submit: Option<extern "C" fn(*mut rtos_work_t)>,

    /* ★design.md P2-2：工作队列统计 out3 = [submitted, soft_overrun, hard_overrun]。 */
    pub work_stats: Option<extern "C" fn(*mut u32)>,
    /* design.md 5#4: per-queue bandwidth quota */
    pub workq_set_quota: Option<extern "C" fn(u8, u32)>,

    /* ★design.md P2-2f：多队列（各独立 worker/优先级）*/ 
    pub workq_create: Option<extern "C" fn(u8, *const c_char, u8, *mut u8, usize)>,
    pub work_submit_q: Option<extern "C" fn(u8, *mut rtos_work_t)>,
    /// ★design.md §5：注册周期 WorkItem（period_cycles=0 注销）。
    pub workq_add_periodic: Option<extern "C" fn(u8, *mut rtos_work_t, u32)>,
    /// ★design.md §5：内核实测 cycles/ms（预算/耗时换算的唯一来源）。
    pub cycles_per_ms: Option<extern "C" fn() -> u32>,

    /* ★design.md P2-3：软件定时器（"定时器→队列桥"）*/
    pub timer_init: Option<extern "C" fn(*mut rtos_timer_t, *const c_char, Option<rtos_timer_cb_t>, *mut c_void)>,
    pub timer_start_ticks: Option<extern "C" fn(*mut rtos_timer_t, i32, u32)>,
}

/* 系统在固定链接地址定义的实例；App 经 extern 引用，不可自行定义。 */
extern "C" {
    pub static mut g_app_slot: app_slot_t;
}

/// 取 g_app_slot 只读引用（内部使用；App 一般只经本 SDK 的 rtos/device/log 封装调用）。
#[inline]
pub fn slot() -> &'static app_slot_t {
    unsafe { &*core::ptr::addr_of!(g_app_slot) }
}

/// 取 g_app_slot 可变裸指针（内部使用；irq.rs 用它在 irq_reg[] 槽位登记 App ISR，
/// 使 app_slot 侧的 irq_enable/irq_disable 能按 irq_id 找到回调）。
#[inline]
pub fn slot_mut() -> *mut app_slot_t {
    core::ptr::addr_of_mut!(g_app_slot)
}
