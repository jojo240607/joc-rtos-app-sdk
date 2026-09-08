//! 驱动私有 ioctl 命令常量：镜像 joc-base `tools/abi/rtos_abi_ioctl.h`
//!（+ `drv/crc.h`、`drv/timer.h`、`drv/exti.h`、`drv/dac.h`、`drv/rtc.h` 等驱动头）。
//! 值与原驱动头严格一致；C 侧变更须同步此处（ABI 版本随 RTOS_ABI_VERSION 提升）。
//!
//! 用法：`device.ioctl(ADC_IOCTL_SET_CHANNEL, &ch as *const u32 as *mut c_void)`。

#![allow(dead_code)]

/* ---- ADC (drv/adc.h) ---- */
pub const ADC_IOCTL_SET_CHANNEL: i32 = 0x01; /* arg: *const u32 channel */
pub const ADC_IOCTL_GET_CHANNEL: i32 = 0x02; /* arg: *mut u32 channel */
pub const ADC_IOCTL_SET_VREF_MV: i32 = 0x03; /* arg: *const u32 vdda_mv */

/* ---- Stream engine (iface/stream_device.h) ---- */
pub const STREAM_IOCTL_SET_MODE: i32 = 0xF0; /* arg: *const stream_xfer_mode_t */
pub const STREAM_IOCTL_GET_MODE: i32 = 0xF1; /* arg: *mut stream_xfer_mode_t */
pub const STREAM_MODE_POLL: u32 = 0;
pub const STREAM_MODE_IRQ: u32 = 1;
pub const STREAM_MODE_DMA: u32 = 2;

/* ---- UART (drv/uart.h) ---- */
pub const UART_IOCTL_SET_BAUDRATE: i32 = 0x01; /* arg: *const u32 baud */
pub const UART_IOCTL_GET_BAUDRATE: i32 = 0x02; /* arg: *mut u32 baud */
pub const UART_IOCTL_GET_BRR: i32 = 0x03;      /* arg: *mut u32 BRR */
pub const UART_IOCTL_SET_FRAMING: i32 = 0x05;  /* arg: *const uart_frame_t */

/* ---- GPIO pin (drv/gpio_pin.h) ---- */
pub const GPIO_IOCTL_TOGGLE: i32 = 0x01; /* arg: NULL */

/* ---- PWM (drv/pwm.h) ---- */
pub const PWM_IOCTL_SET_DUTY_PERCENT: i32 = 0x20; /* arg: *const i32 (0..100) */
pub const PWM_IOCTL_SET_DUTY_TICKS: i32 = 0x21;   /* arg: *const u32 (0..period) */
pub const PWM_IOCTL_SET_FREQ: i32 = 0x22;         /* arg: *const u32 Hz (independent) */
pub const PWM_IOCTL_GET_PERIOD_TICKS: i32 = 0x23; /* arg: *mut u32 (ARR+1) */
pub const PWM_IOCTL_GET_DUTY_TICKS: i32 = 0x24;   /* arg: *mut u32 (current CCR) */
pub const PWM_IOCTL_ENABLE_CHANNEL: i32 = 0x25;   /* arg: NULL */
pub const PWM_IOCTL_DISABLE_CHANNEL: i32 = 0x26;  /* arg: NULL */
pub const PWM_IOCTL_GET_BDTR: i32 = 0x27;         /* arg: *mut u32 */

/* ---- SPI (drv/spi.h) ---- */
pub const SPI_IOCTL_XFER: i32 = 0x40;    /* arg: *mut spi_xfer_t */
pub const SPI_IOCTL_GET_CR1: i32 = 0x41; /* arg: *mut u32 */

/* ---- I2C (drv/i2c.h) ---- */
pub const I2C_IOCTL_MASTER_WRITE: i32 = 0x30; /* arg: *mut i2c_xfer_t */
pub const I2C_IOCTL_MASTER_READ: i32 = 0x31;  /* arg: *mut i2c_xfer_t */
pub const I2C_IOCTL_BUS_SCAN: i32 = 0x32;     /* arg: *mut i2c_scan_t */
pub const I2C_IOCTL_SET_SPEED: i32 = 0x33;    /* arg: *const u32 */
pub const I2C_IOCTL_GET_CR1: i32 = 0x35;      /* arg: *mut u32 */
pub const I2C_IOCTL_GET_BUSY: i32 = 0x36;     /* arg: *mut u32 */
pub const I2C_IOCTL_GET_CCR: i32 = 0x37;      /* arg: *mut u32 */
pub const I2C_IOCTL_GET_CR2_FREQ: i32 = 0x38; /* arg: *mut u32 */
pub const I2C_IOCTL_SET_ADDR: i32 = 0x39;     /* arg: *const u16 */
pub const I2C_IOCTL_GET_ADDR: i32 = 0x3a;     /* arg: *mut u16 */

/* ---- Temperature sensor (drv/temp_sensor.h) ---- */
pub const TEMP_IOCTL_READ_X10: i32 = 0x01;    /* arg: *mut i32 t_x10 */
pub const TEMP_IOCTL_SET_VREF_MV: i32 = 0x02; /* arg: *const u32 */
pub const TEMP_IOCTL_GET_CAL1: i32 = 0x03;    /* arg: *mut u16 */
pub const TEMP_IOCTL_GET_CAL2: i32 = 0x04;    /* arg: *mut u16 */

/* ---- Timer (drv/timer.h) ---- */
pub const TIMER_IOCTL_GET_OVERFLOWS: i32 = 0x01;  /* arg: *mut u32 */
pub const TIMER_IOCTL_GET_COUNTER: i32 = 0x02;    /* arg: *mut u32 */
pub const TIMER_IOCTL_SET_REPETITION: i32 = 0x03; /* arg: *const u32 */
pub const TIMER_IOCTL_GET_REPETITION: i32 = 0x04; /* arg: *mut u32 */
pub const TIMER_IOCTL_ENABLE: i32 = 0x05;         /* arg: NULL */
pub const TIMER_IOCTL_DISABLE: i32 = 0x06;        /* arg: NULL */

/* ---- CRC (drv/crc.h) ---- */
pub const CRC_IOCTL_RESET: i32 = 0x60;  /* arg: none — start a new message */
pub const CRC_IOCTL_UPDATE: i32 = 0x61; /* arg: *const u32 (one word) */
pub const CRC_IOCTL_RESULT: i32 = 0x62; /* arg: *mut u32 (current CRC) */

/* ---- EXTI (drv/exti.h) ---- */
pub const EXTI_IOCTL_GET_COUNT: i32 = 0x30; /* arg: *mut u32 */
pub const EXTI_IOCTL_TRIGGER: i32 = 0x31;   /* arg: NULL — software trigger */

/* ---- DAC (drv/dac.h) ---- */
pub const DAC_IOCTL_SET_VALUE: i32 = 0x60; /* arg: *const u16 (12-bit value) */
pub const DAC_IOCTL_GET_VALUE: i32 = 0x61; /* arg: *mut u16 (read DOR) */
pub const DAC_IOCTL_GET_CR: i32 = 0x62;    /* arg: *mut u32 */

/* ---- RTC (drv/rtc.h)：ioctl 传 rtc_time_t / rtc_date_t ---- */
pub const RTC_IOCTL_SET_TIME: i32 = 0x60;   /* arg: *const rtc_time_t (24h) */
pub const RTC_IOCTL_GET_TIME: i32 = 0x61;   /* arg: *mut rtc_time_t */
pub const RTC_IOCTL_SET_DATE: i32 = 0x62;   /* arg: *const rtc_date_t */
pub const RTC_IOCTL_GET_DATE: i32 = 0x63;   /* arg: *mut rtc_date_t */
pub const RTC_IOCTL_GET_PRER: i32 = 0x64;   /* arg: *mut rtc_prer_t */
pub const RTC_IOCTL_GET_BDCR: i32 = 0x65;   /* arg: *mut u32 */
pub const RTC_IOCTL_GET_RAW_DR: i32 = 0x67; /* arg: *mut u32 */

/* ---- RNG (drv/rng.h)：只读，无 ioctl ---- */

/* ---- Clock (drv/clock.h) ---- */
pub const CLK_IOCTL_GET_SYSCLK_HZ: i32 = 0x01; /* arg: *mut u32 */

/* ---- USB CDC (drv/usb.h) ---- */
pub const USB_IOCTL_GET_GINTSTS: i32 = 0xD0;
pub const USB_IOCTL_GET_GCCFG: i32 = 0xD1;
pub const USB_IOCTL_GET_DSTS: i32 = 0xD2;
pub const USB_IOCTL_GET_ADDRESS: i32 = 0xD3;
pub const USB_IOCTL_CONNECTED: i32 = 0xD4;
pub const USB_IOCTL_SET_LINE_CODING: i32 = 0xD5;
pub const USB_IOCTL_GET_LINE_CODING: i32 = 0xD6;
pub const USB_IOCTL_RUN_CTRL_SELFTEST: i32 = 0xD7;
pub const USB_IOCTL_DBG_DUMP: i32 = 0xD8;
pub const USB_IOCTL_DBG_SET: i32 = 0xD9;
pub const USB_IOCTL_SET_DAD_TEST: i32 = 0xDA;
pub const USB_IOCTL_TX_FREE: i32 = 0xDB;
pub const USB_IOCTL_TX_PUMP: i32 = 0xDC;
pub const USB_IOCTL_RX_REARM: i32 = 0xDD;
pub const USB_IOCTL_REENUM: i32 = 0xDE;
