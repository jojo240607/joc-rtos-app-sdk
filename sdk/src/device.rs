//! RTOS 设备抽象层：安全封装 `g_app_slot.dev_*` vtable。
//! App 经此访问系统已注册的驱动（uart0/usb0/adc0/...），不碰裸寄存器。

use core::ffi::{c_char, c_void};

use crate::abi::{device_t, slot};

/// 设备句柄（opaque `device_t*`）。
///
/// 注意：只 `get` 查找 + `read`/`write`，**绝不 open/close**：uart0 已由 C 侧
/// `g_console` 打开，若 App close 会 deinit 共享 UART 导致 C 侧 printf 冻结。
#[derive(Clone, Copy)]
pub struct Device {
    dev: *mut device_t,
    pub(crate) opened: bool,
}

// 单核 RTOS 下由调用方保证设备访问互斥；句柄本身仅持指针，手工标注。
unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Device {
    /// 按名字查找设备（如 "uart0" / "usb0"）。找不到返回 None。
    pub fn get(name: &str) -> Option<Device> {
        let mut buf = [0u8; 32];
        if name.len() >= buf.len() {
            return None;
        }
        buf[..name.len()].copy_from_slice(name.as_bytes());
        buf[name.len()] = 0;
        let get = slot().dev_get?; // None → 服务表未初始化
        let p = get(buf.as_ptr() as *const c_char);
        if p.is_null() {
            None
        } else {
            Some(Device { dev: p, opened: false })
        }
    }

    /// 打开设备（App 自有驱动；控制台 uart0 除外，见模块注释）。
    pub fn open(name: &str) -> Option<Device> {
        let mut d = Device::get(name)?;
        let rc = d.open_dev();
        if rc == 0 {
            d.opened = true;
            Some(d)
        } else {
            None
        }
    }

    /// 对已 get 到的设备执行 open（返回 vtable open 的 rc）。
    pub fn open_dev(&mut self) -> i32 {
        unsafe {
            let vt = (*self.dev).vtable;
            if (*vt).open.is_some() {
                (*vt).open.unwrap()(self.dev as *mut c_void)
            } else {
                -1
            }
        }
    }

    /// 读（流式设备按当前可用数据返回字节数，可能 < len）。
    pub fn read(&self, buf: &mut [u8]) -> i32 {
        unsafe {
            let vt = (*self.dev).vtable;
            if (*vt).read.is_some() {
                (*vt).read.unwrap()(self.dev as *mut c_void, buf.as_mut_ptr() as *mut c_void, buf.len())
            } else {
                -1
            }
        }
    }

    /// 写（阻塞/有界超时语义由驱动实现；返回实际写入字节数）。
    pub fn write(&self, buf: &[u8]) -> i32 {
        unsafe {
            let vt = (*self.dev).vtable;
            if (*vt).write.is_some() {
                (*vt).write.unwrap()(self.dev as *mut c_void, buf.as_ptr() as *const c_void, buf.len())
            } else {
                -1
            }
        }
    }

    /// ioctl（命令码与驱动相关）。
    pub fn ioctl(&self, cmd: i32, arg: *mut c_void) -> i32 {
        unsafe {
            let vt = (*self.dev).vtable;
            if (*vt).ioctl.is_some() {
                (*vt).ioctl.unwrap()(self.dev as *mut c_void, cmd, arg)
            } else {
                -1
            }
        }
    }

    /// 设备名（调试）。
    pub fn name(&self) -> &'static str {
        unsafe {
            let p = (*self.dev).name;
            if p.is_null() {
                ""
            } else {
                let mut len = 0;
                while *p.add(len) != 0 {
                    len += 1;
                }
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(p as *const u8, len))
            }
        }
    }
}
