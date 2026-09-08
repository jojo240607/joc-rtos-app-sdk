//! 应用层日志系统：复用 RTOS 调试控制台 `uart0`（USART1），前缀统一 `R/{L}/{tag}`。
//! App 仅 `get("uart0")` 查找 + `write`，**绝不 open/close**（uart0 已由 C 侧 g_console 打开）。
//!
//! 简单实现：调用方栈上格式化（`core::fmt::write`，no_std 安全），一次 `dev_write` 输出。
//! 注意：`dev_write(uart0)` 走 DMA TX（阻塞于 TC 信号量），若业务任务高频打日志会自阻塞——
//! 高频/硬实时场景请照搬 joc-app-rust 的「无锁 ring + log_task 消费者」模式。

use core::fmt;
use core::fmt::Write as _;

use crate::device::Device;

/* ---- 日志级别 ---- */
pub const LEVEL_DEBUG: u8 = b'D';
pub const LEVEL_INFO: u8 = b'I';
pub const LEVEL_WARN: u8 = b'W';
pub const LEVEL_ERROR: u8 = b'E';

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

/// 把一条日志写到 uart0 控制台。`args` 由 `info!`/`warn!`/`error!` 宏传入。
pub fn log(level: u8, tag: &str, args: fmt::Arguments<'_>) {
    // 输出格式：R/I/{tag}: {msg}\r\n
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

/* ---- 宏：info!(tag, "...{}...", args) ---- */
#[macro_export]
macro_rules! info {
    ($tag:expr, $($arg:tt)*) => {
        $crate::log::log($crate::log::LEVEL_INFO, $tag, ::core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($tag:expr, $($arg:tt)*) => {
        $crate::log::log($crate::log::LEVEL_WARN, $tag, ::core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($tag:expr, $($arg:tt)*) => {
        $crate::log::log($crate::log::LEVEL_ERROR, $tag, ::core::format_args!($($arg)*))
    };
}
