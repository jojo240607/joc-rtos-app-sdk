//! RTOS 设备抽象层：安全封装 `g_app_slot.dev_*` vtable。
//! App 经此访问系统已注册的驱动（uart0/usb0/adc0/...），不碰裸寄存器。
//! 另含总线传输辅助（`I2cXfer`/`SpiXfer`/`i2c_write_read`），供传感器驱动组合
//! I2C/SPI 事务（布局对齐 RTOS `drv/i2c.h` / `drv/spi.h`）。

use core::ffi::{c_char, c_void};

use crate::abi::{device_t, slot};
use crate::ioctl;

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

/* ===========================================================================
 * 总线传输辅助（I2C / SPI）
 * ========================================================================= */

/// I2C 传输描述符：布局须与 RTOS `drv/i2c.h` 的 `i2c_xfer_t` 一致。
#[repr(C)]
pub struct I2cXfer {
    pub addr: u16,    // 7-bit 从机地址
    pub buf: *mut u8, // 数据缓冲
    pub len: u16,
    pub result: i32, // OUT: 0=ACK, -1=NACK/timeout
}

/// SPI 传输描述符：布局须与 RTOS `drv/spi.h` 的 `spi_xfer_t` 一致。
#[repr(C)]
pub struct SpiXfer {
    pub tx_buf: *const u8, // NULL = 发 0xFF
    pub rx_buf: *mut u8,   // NULL = 丢弃
    pub len: u16,
}

/// I2C 写单个寄存器后读 N 字节（标准 sensor 事务）。
///
/// 调用方需保证 `buf` 生命周期覆盖两次 ioctl，且 `dev` 已 open。
pub fn i2c_write_read(dev: &Device, addr: u16, reg: u8, buf: &mut [u8]) -> i32 {
    // 1) 写寄存器地址
    let mut tx = [reg];
    let mut w = I2cXfer { addr, buf: tx.as_mut_ptr(), len: 1, result: 0 };
    let r = dev.ioctl(ioctl::I2C_IOCTL_MASTER_WRITE, &mut w as *mut I2cXfer as *mut c_void);
    if r != 0 || w.result != 0 {
        return -1;
    }
    // 2) 读数据
    let mut rd = I2cXfer { addr, buf: buf.as_mut_ptr(), len: buf.len() as u16, result: 0 };
    let r = dev.ioctl(ioctl::I2C_IOCTL_MASTER_READ, &mut rd as *mut I2cXfer as *mut c_void);
    if r != 0 || rd.result != 0 {
        return -1;
    }
    0
}
