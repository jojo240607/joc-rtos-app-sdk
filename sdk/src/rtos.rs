//! RTOS 同步原语 + 任务创建 helper 的 Rust 薄封装。
//!
//! 关键：所有 RTOS 内核服务都经 `g_app_slot` vtable 的函数指针间接调用
//! （系统区固件在 APP_SLOT_RAM 填好），**绝不**直接链接 extern "C" 符号——
//! 应用分区独立链接时没有 RTOS 实现，直接 extern 调用会链接失败。

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use crate::abi::{slot, rtos_sem_t, rtos_task_attr_t, rtos_task_entry_t};

/* ===========================================================================
 * 事件信号量（计数初值 0、上限 1）
 * ========================================================================= */
pub struct Semaphore {
    sem: rtos_sem_t,
}

// rtos_sem_t 含裸指针，Rust 视为 !Sync/!Send；RTOS 单核多任务下手工保证只通过地址访问，
// 锁语义保证互斥，故显式标注。
unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}

impl Semaphore {
    /// 编译期零初始化的未初始化信号量；必须先 `init()` 才能使用。
    pub const fn uninit() -> Self {
        Semaphore {
            sem: rtos_sem_t {
                count: 0,
                limit: 0,
                waitq: null_mut(),
            },
        }
    }

    /// 运行时初始化（只调一次）：计数初值 0、上限 1。
    /// 注意：只做 `*const → *mut` 指针转换（不创建 Rust 的 `&mut`），规避 static mut 别名 UB。
    pub fn init(&self) {
        if let Some(f) = slot().sem_init {
            f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t, 0, 1);
        }
    }

    /// 阻塞等待事件（计数 0→1 即返回并清零；1→0）。
    #[inline]
    pub fn wait(&self) {
        if let Some(f) = slot().sem_wait {
            f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t);
        }
    }

    /// 非阻塞尝试获取（0=成功取走一个计数；-1=暂无计数）。
    #[inline]
    pub fn trywait(&self) -> i32 {
        match slot().sem_trywait {
            Some(f) => f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t),
            None => -1,
        }
    }

    /// 投递事件（0→1；已为 1 则 no-op，密集事件合并）。
    #[inline]
    pub fn give(&self) {
        if let Some(f) = slot().sem_give {
            f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t);
        }
    }

    /// 诊断：返回当前计数（确认 init 生效）。
    pub fn debug_count(&self) -> u32 {
        unsafe { (*core::ptr::addr_of!(self.sem)).count }
    }
}

/* ===========================================================================
 * 互斥量（基于 RTOS 二值信号量，初值 1）
 * ========================================================================= */
pub struct Mutex {
    sem: rtos_sem_t,
}

unsafe impl Sync for Mutex {}
unsafe impl Send for Mutex {}

impl Mutex {
    /// 编译期零初始化的未初始化互斥量；必须先 `init()` 才能使用。
    pub const fn uninit() -> Self {
        Mutex {
            sem: rtos_sem_t {
                count: 0,
                limit: 0,
                waitq: null_mut(),
            },
        }
    }

    /// 运行时初始化（只调一次）：二值信号量初值 1、上限 1。
    pub fn init(&self, _ceil_prio: u8) {
        if let Some(f) = slot().sem_init {
            f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t, 1, 1);
        }
    }

    #[inline]
    pub fn lock(&self) {
        if let Some(f) = slot().sem_wait {
            f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t);
        }
    }

    #[inline]
    pub fn unlock(&self) {
        if let Some(f) = slot().sem_give {
            f(&self.sem as *const rtos_sem_t as *mut rtos_sem_t);
        }
    }

    /// RAII 守卫：离开作用域自动解锁。
    pub fn guard(&self) -> MutexGuard<'_> {
        self.lock();
        MutexGuard { m: self }
    }

    /// 诊断：返回当前 sem 计数（确认 init 是否生效；init 成功后应为 1）。
    pub fn debug_count(&self) -> u32 {
        unsafe { (*core::ptr::addr_of!(self.sem)).count }
    }
}

pub struct MutexGuard<'a> {
    m: &'a Mutex,
}

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        self.m.unlock();
    }
}

/* ===========================================================================
 * 基础服务
 * ========================================================================= */

/// 经 g_app_slot 的 RTOS msleep（SysTick 1000Hz 驱动调度）。
#[inline]
pub fn msleep(ms: u32) {
    if let Some(f) = slot().msleep {
        f(ms);
    }
}

/// 经 g_app_slot 的 RTOS tick_count（系统启动后 tick 数）。
#[inline]
pub fn tick_count() -> u32 {
    if let Some(f) = slot().tick_count {
        f()
    } else {
        0
    }
}

/// 经 g_app_slot 的 RTOS cycle_now（DWT CYCCNT，高精度周期计数）。
#[inline]
pub fn cycle_now() -> u32 {
    if let Some(f) = slot().cycle_now {
        f()
    } else {
        0
    }
}

/// 经 g_app_slot 的 RTOS 绝对延时（FreeRTOS vTaskDelayUntil 语义）。
///
/// 睡到 `*last + inc_ticks` 时刻并推进 `*last`：内核把【目标时刻 − 当前时刻】
/// 写进睡眠队列，tick ISR 在目标时刻直接 ready 本任务。因此无论任务自身执行
/// 时间多长、被抢占多久，**周期恒定不漂移**（相对 msleep 的周期 = sleep + 执行）。
/// 对需要积分的控制/EKF 任务，这是正确的周期语义。已超期则内核重同步，不长眠。
#[inline]
pub fn delay_until(last: &mut u32, inc_ticks: u32) {
    if let Some(f) = slot().delay_until {
        f(last as *mut u32, inc_ticks);
    }
}

/// 创建并启动一个 RTOS 任务（经 g_app_slot 间接调用，App 独立链接安全）。
///
/// - `name`：以 `\0` 结尾的 ASCII 名（内部自动补 0，传 "name" 即可）。
/// - `entry`：`extern "C" fn(*mut c_void)` 入口。
/// - `prio`：优先级档位。
/// - `stack`/`stack_size`：调用方提供的静态栈缓冲（`#[link_section = ".rust_bss"]`）。
/// - `priv_`：1=特权（飞控关键任务推荐）。
/// - `rt_class`：RTOS_RT_NONE / RTOS_RT_HARD / RTOS_RT_SOFT（HARD 时 prio ≤ RTOS_PRIO_BH_HIGH）。
/// - `deadline_ticks`/`wcet_ticks`：硬实时最坏期限/预算（0=无）。
pub fn spawn_rt(
    name: &str,
    entry: rtos_task_entry_t,
    prio: u8,
    stack: *mut u8,
    stack_size: usize,
    priv_: u8,
    rt_class: u8,
    deadline_ticks: u32,
    wcet_ticks: u32,
) {
    if let Some(f) = slot().task_create_rt {
        let mut nbuf = [0u8; 24];
        let n = name.len().min(nbuf.len() - 1);
        nbuf[..n].copy_from_slice(&name.as_bytes()[..n]);
        nbuf[n] = 0;
        let attr = rtos_task_attr_t {
            rt_class,
            deadline_ticks,
            wcet_ticks,
        };
        f(
            nbuf.as_ptr() as *const c_char,
            entry,
            null_mut(),
            prio,
            stack as *mut c_void,
            stack_size,
            priv_,
            &attr as *const rtos_task_attr_t,
        );
    }
}

/// 便捷：创建普通（非硬实时）任务。
pub fn spawn(
    name: &str,
    entry: rtos_task_entry_t,
    prio: u8,
    stack: *mut u8,
    stack_size: usize,
) {
    spawn_rt(name, entry, prio, stack, stack_size, 1, RTOS_RT_NONE, 0, 0);
}

// 导出常用常量，方便 App 侧直接 `use rtos_app_sdk::rtos::*`。
pub use crate::abi::{
    RTOS_PRIO_BH_HIGH, RTOS_PRIO_BH_MED, RTOS_PRIO_BLINK, RTOS_PRIO_IDLE, RTOS_PRIO_MAIN,
    RTOS_RT_HARD, RTOS_RT_NONE, RTOS_RT_SOFT,
};
