//! 应用层日志系统：复用 RTOS 调试控制台 `uart0`（USART1），前缀统一 `R/{L}/{tag}`。
//! App 仅 `get("uart0")` 查找 + `write`，**绝不 open/close**（uart0 已由 C 侧 g_console 打开）。
//!
//! 两档实现：
//!   - **同步直写**（`log`/`raw`）：调用方栈上格式化，一次 `dev_write` 输出。适用于低
//!     频/非实时场景（如 demo 应用）。注意 `dev_write(uart0)` 走 DMA TX（阻塞于 TC
//!     信号量），业务任务高频打日志会自阻塞。
//!   - **无锁 ring + log_task 消费者**（`emit`/`spawn_log_task`）：生产者只把格式化日志
//!     写入无锁环形缓冲（原子头尾 + 非阻塞互斥，拿不到锁丢本次），绝不阻塞业务任务；
//!     独立低优先任务 `log_task`（prio 28）周期 drain ring → uart0 一次拼帧输出。
//!     高频/硬实时场景（飞控）应调用一次 `spawn_log_task()` 后使用宏。
//!     `emit` 在 log_task 未启动时自动回退同步直写，保证日志不丢。

use core::fmt;
use core::fmt::Write as _;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::device::Device;
use crate::abi::RTOS_RT_NONE;
use crate::rtos::{msleep, spawn_rt, tick_count};

/* ---- 日志级别 ---- */
pub const LEVEL_DEBUG: u8 = b'D';
pub const LEVEL_INFO: u8 = b'I';
pub const LEVEL_WARN: u8 = b'W';
pub const LEVEL_ERROR: u8 = b'E';

/// 该级别是否"关键"（关键条目不被 Info 覆盖）。
const fn critical(level: u8) -> bool {
    matches!(level, LEVEL_WARN | LEVEL_ERROR)
}

/// 栈上格式化 writer。
struct BufWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> BufWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        BufWriter { buf, len: 0 }
    }
    fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl fmt::Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let room = self.buf.len() - self.len;
        let n = room.min(bytes.len());
        self.buf[self.len..self.len + n].copy_from_slice(&bytes[..n]);
        self.len += n;
        Ok(())
    }
}

/// 把 u32 写成十进制 ASCII，返回写入字节数。
fn write_u32(dst: &mut [u8], mut v: u32) -> usize {
    if v == 0 {
        if !dst.is_empty() {
            dst[0] = b'0';
            return 1;
        }
        return 0;
    }
    let mut tmp = [0u8; 10];
    let mut i = 0;
    while v > 0 && i < tmp.len() {
        tmp[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }
    let mut n = 0;
    while i > 0 {
        i -= 1;
        if n < dst.len() {
            dst[n] = tmp[i];
            n += 1;
        }
    }
    n
}

/* ===========================================================================
 * 同步直写（简单版）
 * ========================================================================= */

/// 把一条日志写到 uart0 控制台。`args` 由 `info!`/`warn!`/`error!` 宏传入。
pub fn log(level: u8, tag: &str, args: fmt::Arguments<'_>) {
    // 输出格式：R/{L}/{tag}: {msg}\r\n
    let mut buf = [0u8; 256];
    let mut w = BufWriter::new(&mut buf);
    let _ = w.write_str("R/");
    let _ = w.write_str(match level {
        LEVEL_DEBUG => "D",
        LEVEL_INFO => "I",
        LEVEL_WARN => "W",
        _ => "E",
    });
    let _ = w.write_str("/");
    let _ = w.write_str(tag);
    let _ = w.write_str(": ");
    let _ = fmt::write(&mut w, args);
    let _ = w.write_str("\r\n");
    // 直接写控制台 uart0。
    if let Some(uart0) = Device::get("uart0") {
        let _ = uart0.write(w.as_slice());
    }
}

/// 直接输出一行原始日志（无 tag/前缀，如生命周期自报）。
pub fn raw(level: u8, text: &str) {
    let mut buf = [0u8; 256];
    let mut w = BufWriter::new(&mut buf);
    let _ = w.write_str(match level {
        LEVEL_DEBUG => "R/D: ",
        LEVEL_INFO => "R/I: ",
        LEVEL_WARN => "R/W: ",
        _ => "R/E: ",
    });
    let _ = w.write_str(text);
    let _ = w.write_str("\r\n");
    if let Some(uart0) = Device::get("uart0") {
        let _ = uart0.write(w.as_slice());
    }
}

/* ===========================================================================
 * 无锁环形缓冲（单生产者写 / 单消费者读，原子头尾 + 非阻塞互斥）
 * ========================================================================= */

const LOG_RING_SIZE: usize = 2048; // 静态 ring，放 App RAM

#[link_section = ".rust_bss"]
static mut LOG_RING: [u8; LOG_RING_SIZE] = [0u8; LOG_RING_SIZE];
#[link_section = ".rust_bss"]
static LOG_HEAD: AtomicU32 = AtomicU32::new(0); // 消费者读位置
#[link_section = ".rust_bss"]
static LOG_TAIL: AtomicU32 = AtomicU32::new(0); // 生产者写位置

/// 日志写互斥（非阻塞 trywait；拿不到就丢本次日志）。
#[link_section = ".rust_bss"]
static LOG_MTX: AtomicBool = AtomicBool::new(false);

fn try_lock() -> bool {
    LOG_MTX
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
}

fn unlock() {
    LOG_MTX.store(false, Ordering::Release);
}

/// 在 ring 中写入一条 `[len][level][payload]` 日志。head/tail 均为字节偏移。
/// 满时策略：只允许覆盖 Info 级旧条目（跳过 Warn/Error），Warn/Error 不覆盖。
fn ring_push(level: u8, payload: &[u8]) -> bool {
    if payload.is_empty() || payload.len() > 255 {
        return false;
    }
    let ring: &mut [u8; LOG_RING_SIZE] = unsafe { &mut LOG_RING };
    let head = LOG_HEAD.load(Ordering::Relaxed) as usize;
    let tail = LOG_TAIL.load(Ordering::Relaxed) as usize;
    let need = payload.len() + 2;
    if need >= LOG_RING_SIZE {
        return false;
    }
    let free = if tail >= head { LOG_RING_SIZE - (tail - head) } else { head - tail };
    if free >= need {
        let n = payload.len();
        if n + 1 > u8::MAX as usize {
            return false;
        }
        ring[tail] = (n + 1) as u8; // len（不含 len 字节，含 level）
        ring[(tail + 1) % LOG_RING_SIZE] = level;
        for (i, &b) in payload.iter().enumerate() {
            ring[(tail + 2 + i) % LOG_RING_SIZE] = b;
        }
        LOG_TAIL.store(((tail + need) % LOG_RING_SIZE) as u32, Ordering::Release);
        return true;
    }
    // ring 满。关键级别(Warn/Error)：跳过覆盖（丢本次），保存量关键日志。
    if critical(level) {
        return false;
    }
    // Info：向前覆盖最旧的非关键(Info)条目，直到腾出空间或只剩关键条目。
    let mut h = head;
    let mut scanned = 0usize;
    while free + scanned < need && scanned < LOG_RING_SIZE {
        let cur = h;
        let cur_len = ring[cur] as usize + 1;
        let cur_level = ring[(cur + 1) % LOG_RING_SIZE];
        if cur_level == LEVEL_WARN || cur_level == LEVEL_ERROR {
            return false; // 关键条目不覆盖 → 放弃本次 Info
        }
        h = (h + cur_len) % LOG_RING_SIZE;
        scanned += cur_len;
    }
    if free + scanned < need {
        return false;
    }
    LOG_HEAD.store(h as u32, Ordering::Release);
    let tail = LOG_TAIL.load(Ordering::Relaxed) as usize;
    let n = payload.len();
    ring[tail] = (n + 1) as u8;
    ring[(tail + 1) % LOG_RING_SIZE] = level;
    for (i, &b) in payload.iter().enumerate() {
        ring[(tail + 2 + i) % LOG_RING_SIZE] = b;
    }
    LOG_TAIL.store(((tail + need) % LOG_RING_SIZE) as u32, Ordering::Release);
    true
}

/// 从 ring 读取一条条目（含 len/level 头），前移 head。返回读取字节数（0=空）。
fn ring_pop(out: &mut [u8]) -> usize {
    let ring: &mut [u8; LOG_RING_SIZE] = unsafe { &mut LOG_RING };
    let head = LOG_HEAD.load(Ordering::Acquire) as usize;
    let tail = LOG_TAIL.load(Ordering::Acquire) as usize;
    if head == tail {
        return 0;
    }
    let first = ring[head];
    let entry_len = (first as usize) + 1;
    let n = entry_len.min(out.len());
    for i in 0..n {
        out[i] = ring[(head + i) % LOG_RING_SIZE];
    }
    LOG_HEAD.store(((head + entry_len) % LOG_RING_SIZE) as u32, Ordering::Release);
    n
}

/* ===========================================================================
 * emit：生产者入口。只写 ring，绝不阻塞、绝不直接写串口。
 * log_task 未启动（无消费者）时回退同步直写，保证日志不丢。
 * ========================================================================= */

/// log_task 是否已启动（spawn_log_task 置位）。未启动时 emit 回退同步直写。
#[link_section = ".rust_bss"]
static LOG_TASK_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn emit(level: u8, tag: &str, args: fmt::Arguments) {
    if !LOG_TASK_ACTIVE.load(Ordering::Acquire) {
        // 无消费者：同步直写（低频场景）。
        log(level, tag, args);
        return;
    }
    // 非阻塞互斥：拿不到锁（别的任务正在写）→ 丢弃本次日志（日志可丢）。
    if !try_lock() {
        return;
    }
    let mut buf = [0u8; 180];
    let mut len = 0usize;

    buf[len] = b'R'; len += 1;
    buf[len] = b'/'; len += 1;
    buf[len] = level; len += 1;
    buf[len] = b' '; len += 1;

    len += write_u32(&mut buf[len..], tick_count());
    buf[len] = b' '; len += 1;

    let tb = tag.as_bytes();
    let n = tb.len().min(buf.len() - len - 4);
    buf[len..len + n].copy_from_slice(&tb[..n]);
    len += n;
    buf[len] = b':'; len += 1;
    buf[len] = b' '; len += 1;

    let mut w = BufWriter::new(&mut buf[len..]);
    let _ = fmt::write(&mut w, args);
    len += w.len;

    // 换行语义归一：ring 内只存 `\n`，由消费者(log_task)拼包时统一转 `\r\n`。
    buf[len] = b'\n'; len += 1;

    ring_push(level, &buf[..len]);
    unlock();
}

/* ===========================================================================
 * log_task：消费者。低优先级，周期 drain ring → uart0 输出。
 * ========================================================================= */

/// 日志任务栈（放 App RAM）。容量必须足够：聚合缓冲 pkt(512B) + 单条解码 tmp(256B)
/// + ring_pop/dev.write/msleep 栈帧。早期 1024B 太小曾向下溢出覆盖 PLAYBACK 全局，
/// 改为 4096 提供充足余量。
#[link_section = ".rust_bss"]
static mut LOG_TASK_STACK: [u8; 4096] = [0u8; 4096];

/// 日志任务优先级：低于所有业务任务（ctrl=4,sensor=5,uplink=10,telem=12），
/// 靠近 idle(31)，即使阻塞在 uart0 也不影响业务。取 28。
const LOG_TASK_PRIO: u8 = 28;

extern "C" fn log_task_entry(_arg: *mut core::ffi::c_void) {
    // uart0 复用 C 侧已打开的句柄；只 get + write，绝不 open/close。
    let uart = Device::get("uart0\0");
    let mut pkt = [0u8; 512];
    let mut pkt_len = 0usize;
    loop {
        let mut flushed = false;
        while let Some(dev) = uart.as_ref() {
            // 互斥地取走一条 `[len][level][payload]`（payload 末尾为 `\n`）。
            let entry = {
                if !try_lock() {
                    break;
                }
                let mut tmp = [0u8; 256];
                let got = ring_pop(&mut tmp);
                unlock();
                if got == 0 {
                    break; // ring 空
                }
                tmp
            };
            let e_len = entry[0] as usize;
            // 防御：len 字段非法（0 或越界）时丢弃坏条目，绝不 panic 挂死 log_task。
            if e_len == 0 || e_len + 1 > entry.len() {
                break;
            }
            let payload = &entry[1..e_len + 1]; // 去掉 len 头，保留 level+payload(含 \n)

            let mut i = 0usize;
            while i < payload.len() {
                if payload[i] == b'\n' {
                    if pkt_len + 2 > pkt.len() {
                        break;
                    }
                    pkt[pkt_len] = b'\r';
                    pkt[pkt_len + 1] = b'\n';
                    pkt_len += 2;
                } else {
                    if pkt_len + 1 > pkt.len() {
                        break;
                    }
                    pkt[pkt_len] = payload[i];
                    pkt_len += 1;
                }
                i += 1;
            }
            if i < payload.len() {
                let _ = dev.write(&pkt[..pkt_len]);
                pkt_len = 0;
                flushed = true;
                while i < payload.len() {
                    if payload[i] == b'\n' {
                        if pkt_len + 2 > pkt.len() {
                            break;
                        }
                        pkt[pkt_len] = b'\r';
                        pkt[pkt_len + 1] = b'\n';
                        pkt_len += 2;
                    } else {
                        if pkt_len + 1 > pkt.len() {
                            break;
                        }
                        pkt[pkt_len] = payload[i];
                        pkt_len += 1;
                    }
                    i += 1;
                }
                continue;
            }
            if pkt_len + payload.len() + 2 > pkt.len() {
                let _ = dev.write(&pkt[..pkt_len]);
                pkt_len = 0;
                flushed = true;
            }
        }
        if pkt_len > 0 {
            if let Some(dev) = uart.as_ref() {
                let _ = dev.write(&pkt[..pkt_len]);
            }
            pkt_len = 0;
            flushed = true;
        }
        if !flushed {
            msleep(5); // 本轮什么都没发 → 节流睡眠
        }
    }
}

/// 创建日志任务（高频场景在业务任务拉起前调用一次）。
pub fn spawn_log_task() {
    unsafe {
        spawn_rt(
            "log_task",
            log_task_entry,
            LOG_TASK_PRIO,
            LOG_TASK_STACK.as_mut_ptr(),
            LOG_TASK_STACK.len(),
            1,
            RTOS_RT_NONE,
            0,
            0,
        );
    }
    LOG_TASK_ACTIVE.store(true, Ordering::Release);
}

/* ---- 宏：info!/warn!/error!/debug! 双语法（兼容旧 `tag:` 形式与直接 tag） ---- */
#[macro_export]
macro_rules! info {
    (tag: $tag:expr, $($arg:tt)*) => {
        $crate::log::emit($crate::log::LEVEL_INFO, $tag, ::core::format_args!($($arg)*))
    };
    ($tag:expr, $($arg:tt)*) => {
        $crate::log::emit($crate::log::LEVEL_INFO, $tag, ::core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    (tag: $tag:expr, $($arg:tt)*) => {
        $crate::log::emit($crate::log::LEVEL_WARN, $tag, ::core::format_args!($($arg)*))
    };
    ($tag:expr, $($arg:tt)*) => {
        $crate::log::emit($crate::log::LEVEL_WARN, $tag, ::core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    (tag: $tag:expr, $($arg:tt)*) => {
        $crate::log::emit($crate::log::LEVEL_ERROR, $tag, ::core::format_args!($($arg)*))
    };
    ($tag:expr, $($arg:tt)*) => {
        $crate::log::emit($crate::log::LEVEL_ERROR, $tag, ::core::format_args!($($arg)*))
    };
}

/// `debug!(tag, "msg")` / `debug!(tag: "x", "msg")` —— 仅 debug build 编入（release 剔除）。
#[macro_export]
macro_rules! debug {
    (tag: $tag:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            $crate::log::emit($crate::log::LEVEL_DEBUG, $tag, ::core::format_args!($($arg)*))
        }
    };
    ($tag:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            $crate::log::emit($crate::log::LEVEL_DEBUG, $tag, ::core::format_args!($($arg)*))
        }
    };
}
