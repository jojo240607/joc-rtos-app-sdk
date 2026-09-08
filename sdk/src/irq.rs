//! 中断回调注册：App 经 `g_app_slot` 的 `irq_attach / irq_enable / irq_disable`
//! 把自己的 ISR 挂到指定 IRQ 线。
//!
//! 语义（见 joc-base `app_slot/app_slot.c` + `irq/irq_manager.c`）：
//! - 与系统侧驱动 ISR **并存**：irq_dispatch 会依次调用同一 IRQ 线上的全部处理器
//!   （驱动在 open 时挂一个，App 可再挂一个，互不覆盖）；
//! - **登记表**：App 必须把 `app_irq_reg_t` 写进 `g_app_slot.irq_reg[]`（本模块
//!   自动选空闲槽），`irq_enable/irq_disable` 靠它在 NVIC 上按 irq_id 找回调；
//! - `irq_attach` 只注册 + 设优先级，**不使能**（NVIC 掩码由 `irq_enable` 打开）；
//! - ISR 里只能做最快的事：给信号量 / 置标志 / 计数器++。**禁止调用阻塞 API**
//!   （msleep / sem_wait / dev_write 等）——中断上下文无任务可让出。
//!
//! 用法：
//! ```ignore
//! static mut IRQ_SEM: Semaphore = Semaphore::uninit();
//! extern "C" fn my_isr(_ctx: *mut c_void) { unsafe { IRQ_SEM.give(); } }
//! irq::attach(28 /*TIM2_UP*/, IRQ_CLASS_KERNEL, 0, my_isr, null_mut());
//! irq::enable(28);
//! ```

use core::ffi::c_void;

use crate::abi::{app_irq_reg_t, slot, slot_mut, IRQ_CLASS_NORMAL, APP_IRQ_REG_MAX};

/// 注册 App ISR 到指定 IRQ 线（登记进 g_app_slot.irq_reg[] 空闲槽；不使能；
/// 返回 0=成功，-1=槽位耗尽/线不可用）。
pub fn attach(
    irq_id: u8,
    prio_class: u8, // IRQ_CLASS_NORMAL / IRQ_CLASS_KERNEL / IRQ_CLASS_ZERO_LATENCY
    rt_class: u8,   // 0=普通, 1=硬实时（仅 KERNEL 类有意义）
    isr: extern "C" fn(*mut c_void),
    ctx: *mut c_void,
) -> i32 {
    let f = match slot().irq_attach {
        Some(f) => f,
        None => return -1,
    };
    let reg = app_irq_reg_t {
        used: 1,
        irq_id,
        prio_class,
        rt_class,
        isr_cb: Some(isr),
        ctx,
    };
    unsafe {
        let slotp = slot_mut();
        for i in 0..APP_IRQ_REG_MAX {
            let r = &mut (*slotp).irq_reg[i];
            if r.used == 0 {
                *r = reg;
                return f(r);
            }
        }
    }
    -1 // 8 个槽位耗尽
}

/// 使能（开 NVIC 掩码）已注册的 IRQ 线。
pub fn enable(irq_id: u8) -> i32 {
    match slot().irq_enable {
        Some(f) => f(irq_id),
        None => -1,
    }
}

/// 屏蔽已注册的 IRQ 线。
pub fn disable(irq_id: u8) -> i32 {
    match slot().irq_disable {
        Some(f) => f(irq_id),
        None => -1,
    }
}

/// 便捷：`attach` + `enable` 一步完成（普通优先级类）。
pub fn attach_and_enable(
    irq_id: u8,
    isr: extern "C" fn(*mut c_void),
    ctx: *mut c_void,
) -> i32 {
    let rc = attach(irq_id, IRQ_CLASS_NORMAL, 0, isr, ctx);
    if rc != 0 {
        return rc;
    }
    enable(irq_id)
}
