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

/* ---- CAN (drv/can.h) ---- */
pub const CAN_IOCTL_SEND_FRAME: i32 = 0x01; /* arg: *mut can_frame_t（发一帧） */
pub const CAN_IOCTL_RECV_FRAME: i32 = 0x02; /* arg: *mut can_frame_t（收一帧） */
pub const CAN_IOCTL_GET_MCR: i32 = 0x03;    /* arg: *mut u32 CAN_MCR */
pub const CAN_IOCTL_GET_BTR: i32 = 0x04;    /* arg: *mut u32 CAN_BTR (LBKM/SILM/timing) */
pub const CAN_IOCTL_GET_MSR: i32 = 0x05;    /* arg: *mut u32 CAN_MSR */
pub const CAN_IOCTL_GET_ESR: i32 = 0x06;    /* arg: *mut u32 CAN_ESR */
pub const CAN_IOCTL_GET_TSR: i32 = 0x07;    /* arg: *mut u32 CAN_TSR */
pub const CAN_IOCTL_GET_RF0R: i32 = 0x08;   /* arg: *mut u32 CAN_RF0R */
pub const CAN_IOCTL_GET_FA1R: i32 = 0x09;   /* arg: *mut u32 CAN_FA1R */
pub const CAN_IOCTL_GET_FMR: i32 = 0x0A;    /* arg: *mut u32 CAN_FMR */
pub const CAN_BTR_LBKM: u32 = 1 << 30;      /* 回环模式（自测）；SILM=bit31（F407 CAN_BTR） */

/// CAN 单帧（与 hal/stm32/can_hal.h 的 can_frame_t 布局一致，repr(C)）
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CanFrame {
    pub id: u32,      /* 11 位标准 / 29 位扩展 ID */
    pub ext: u8,      /* 1 = 扩展 29 位 */
    pub rtr: u8,      /* 1 = 远程帧（无数据） */
    pub dlc: u8,      /* 数据长度 0..8 */
    pub data: [u8; 8],
}

/* ---- FLASH (drv/flash.h)：受管扇区管理器 ---- */
pub const FLASH_IOCTL_GET_SECTOR: i32 = 0x70; /* arg: *mut u32 受管扇区号 */
pub const FLASH_IOCTL_GET_BASE: i32 = 0x71;   /* arg: *mut u32 扇区基址 */
pub const FLASH_IOCTL_GET_STATUS: i32 = 0x72; /* arg: *mut u32 原始 FLASH->SR */
pub const FLASH_SR_BSY: u32 = 1 << 16;        /* 忙标志 */

/* ---- IWDG (drv/iwdg.h) ---- */
pub const IWDG_IOCTL_SET_PRESCALER: i32 = 0x60; /* arg: *const u32 (0..7) */
pub const IWDG_IOCTL_SET_RELOAD: i32 = 0x61;    /* arg: *const u32 (0..4095) */
pub const IWDG_IOCTL_GET_PRESCALER: i32 = 0x62; /* arg: *mut u32 */
pub const IWDG_IOCTL_GET_RELOAD: i32 = 0x63;    /* arg: *mut u32 */
pub const IWDG_IOCTL_START: i32 = 0x64;         /* arg: none — ARM（喂狗前会复位） */
pub const IWDG_IOCTL_REFRESH: i32 = 0x65;       /* arg: none — 喂狗 */
pub const IWDG_IOCTL_GET_STATUS: i32 = 0x66;    /* arg: *mut u32 原始 SR */

/* ---- WWDG (drv/wwdg.h) ---- */
pub const WWDG_IOCTL_SET_PRESCALER: i32 = 0x60; /* arg: *const u32 (WDGTB 0..3) */
pub const WWDG_IOCTL_SET_WINDOW: i32 = 0x61;    /* arg: *const u32 (0x40..0x7F) */
pub const WWDG_IOCTL_GET_CONFIG: i32 = 0x62;    /* arg: *mut u32 原始 CFR */
pub const WWDG_IOCTL_START: i32 = 0x63;         /* arg: *const u32 (reload 0x40..0x7F) */
pub const WWDG_IOCTL_REFRESH: i32 = 0x64;       /* arg: *const u32 (reload > window) */
pub const WWDG_IOCTL_GET_COUNTER: i32 = 0x65;   /* arg: *mut u32 原始 CR.T */
pub const WWDG_IOCTL_GET_STATUS: i32 = 0x66;    /* arg: *mut u32 原始 SR */

/* ---- I2S (drv/i2s.h)：i2s0 = I2S2 主机 TX（无外部 codec） ---- */
pub const I2S_IOCTL_GET_I2SCFGR: i32 = 0x01;  /* arg: *mut u32 */
pub const I2S_IOCTL_GET_I2SPR: i32 = 0x02;    /* arg: *mut u32 */
pub const I2S_IOCTL_GET_PLLI2S: i32 = 0x03;   /* arg: *mut u32 RCC_PLLI2SCFGR */
pub const I2S_IOCTL_GET_CFGR: i32 = 0x04;     /* arg: *mut u32 RCC_CFGR (I2SSRC) */
pub const I2S_IOCTL_GET_PLL_RDY: i32 = 0x05;  /* arg: *mut u32 1=PLLI2SRDY */
pub const I2S_IOCTL_GET_AUDIO_HZ: i32 = 0x06; /* arg: *mut u32 请求采样率 */
pub const I2S_IOCTL_GET_I2S_CLK: i32 = 0x07;  /* arg: *mut u32 PLLI2S 输出 Hz */
pub const I2S_I2SCFGR_I2SE: u32 = 1 << 10;    /* I2S 使能位 */

/* ---- SDIO (drv/sdio.h) ---- */
pub const SDIO_IOCTL_CMD: i32 = 0x50;        /* arg: *mut sdio_cmd_t（无数据） */
pub const SDIO_IOCTL_CMD_DATA: i32 = 0x51;   /* arg: *mut sdio_cmd_data_t */
pub const SDIO_IOCTL_SET_CLOCK: i32 = 0x52;  /* arg: *const u32 clkdiv */
pub const SDIO_IOCTL_GET_POWER: i32 = 0x54;  /* arg: *mut u32 */
pub const SDIO_IOCTL_GET_CLKCR: i32 = 0x55;  /* arg: *mut u32 */
pub const SDIO_CLKCR_CLKDIV: u32 = 0xFF;    /* CLKDIV[7:0]（open 置 118） */
pub const SDIO_CLKCR_CLKEN: u32 = 1 << 8;   /* 时钟使能（F4 为 bit8） */
pub const SDIO_CLKCR_WIDBUS_0: u32 = 1 << 11; /* WIDBUS=01（4-bit 总线） */

/* ---- SD Card (drv/sd_card.c)：初始化 + 块读写 ---- */
pub const SD_CARD_IOCTL_INIT: i32 = 0x60; /* arg: none — SD 卡初始化序列 */
pub const SD_CARD_IOCTL_READ_BLOCK: i32 = 0x61;  /* arg: *mut SdBlockIo — 读扇区 */
pub const SD_CARD_IOCTL_WRITE_BLOCK: i32 = 0x62; /* arg: *mut SdBlockIo — 写扇区 */

/// SD 卡块读写参数（与 hal/drv 的 sd_block_io_t 布局一致，repr(C)）
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SdBlockIo {
    pub lba: u64,      /* 起始扇区（512B 块） */
    pub count: u32,    /* 扇区数（1 = 单块 CMD17/24） */
    pub buf: *mut u8,  /* 数据缓冲（count*512 字节） */
}

/* ---- FSMC (drv/fsmc.h)：灵活静态存储器控制器 ---- */
pub const FSMC_IOCTL_SET_BCR: i32 = 0x80;   /* arg: *const u32 BCR1 值 */
pub const FSMC_IOCTL_GET_BCR: i32 = 0x81;   /* arg: *mut u32 BCR1 回读 */
pub const FSMC_IOCTL_SET_BTR: i32 = 0x82;   /* arg: *const u32 BTR1 值 */
pub const FSMC_IOCTL_GET_BTR: i32 = 0x83;   /* arg: *mut u32 BTR1 回读 */
pub const FSMC_IOCTL_GET_BWTR: i32 = 0x84;  /* arg: *mut u32 BWTR1 回读 */
pub const FSMC_IOCTL_BANK1_ENABLE: i32 = 0x85; /* arg: none — BCR1.MBKEN=1 */
pub const FSMC_BCR_MBKEN: u32 = 1 << 0;     /* memory bank enable（窗口可用） */
pub const FSMC_BCR_WREN: u32 = 1 << 12;     /* write enable */
pub const FSMC_BCR_MTYP: u32 = 0x3 << 2;    /* memory type[1:0] */
pub const FSMC_BCR_MWID: u32 = 0x3 << 4;    /* data bus width[1:0] */
