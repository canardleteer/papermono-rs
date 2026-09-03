//! PaperMono-Lite simple-debug image.
//!
//! On the unit: USB-Serial/JTAG prints a repeating hello (image / SKU /
//! clocks / reset), a git line, a GPIO sample, a 1 Hz heartbeat of
//! BUTTON A (UP) / BUTTON B (DOWN), and edge lines on those keys. The
//! e-paper panel does not refresh. No I2C, no NFC, no LoRa, no PDM, no
//! GPIO45/46 latch.
//!
//! In the MCU: blocking `esp-hal`. No Embassy, no RTOS, no panel LUT.
//!
//! `espflash save-image` needs [`esp_bootloader_esp_idf::esp_app_desc`].

#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::clock::{cpu_clock, xtal_clock};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::main;
use esp_hal::rtc_cntl::SocResetReason;
use esp_hal::system::reset_reason;
use esp_println::print;
use m5stack_papermono_lite::pins;
use m5stack_papermono_lite::SKU;
use papermono_log::{
    format_edge, format_git, format_gpio, format_heartbeat, format_hello, Edge, GpioSample, Hello,
    Snapshot, EDGE_CAPACITY, GIT_CAPACITY, GPIO_CAPACITY, HEARTBEAT_CAPACITY, HEARTBEAT_PERIOD_MS,
    HELLO_CAPACITY, HELLO_PERIOD_MS, IMAGE, MILLIS_PER_SEC, POLL_PERIOD_MS,
};

esp_bootloader_esp_idf::esp_app_desc!();

const _: () = {
    assert!(pins::BUTTON_A == 2);
    assert!(pins::BUTTON_B == 3);
    assert!(pins::PMIC_BOOT_OUT == 0);
    assert!(pins::PMIC_IRQ == 1);
    assert!(pins::TOUCH_INT == 4);
    assert!(pins::IOE1_IRQ == 7);
    assert!(pins::EPD_BUSY == 18);
};

/// Map cheap inputs, print the grammar, then poll. Power is the red M5PM1 button.
#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    let btn_a = Input::new(
        peripherals.GPIO2,
        InputConfig::default().with_pull(Pull::Up),
    );
    let btn_b = Input::new(
        peripherals.GPIO3,
        InputConfig::default().with_pull(Pull::Up),
    );
    // No pulls: these nets are not user keys. Expander and panel stay off.
    let boot = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::None),
    );
    let pmic_irq = Input::new(
        peripherals.GPIO1,
        InputConfig::default().with_pull(Pull::None),
    );
    let tp = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(Pull::None),
    );
    let ioe = Input::new(
        peripherals.GPIO7,
        InputConfig::default().with_pull(Pull::None),
    );
    let busy = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::None),
    );

    let hello = Hello {
        t_s: 0,
        image: IMAGE,
        sku: SKU,
        cpu_mhz: cpu_clock().as_mhz(),
        xtal_mhz: xtal_clock().as_mhz(),
        reset: reset_token(reset_reason()),
    };

    let mut t_ms = 0_u32;
    let mut prev_a = btn_a.is_high();
    let mut prev_b = btn_b.is_high();

    loop {
        let now_a = btn_a.is_high();
        let now_b = btn_b.is_high();
        if now_a != prev_a || now_b != prev_b {
            let edge = Edge {
                t_ms,
                btn_a: (now_a != prev_a).then_some((prev_a, now_a)),
                btn_b: (now_b != prev_b).then_some((prev_b, now_b)),
            };
            emit_edge(&edge);
            prev_a = now_a;
            prev_b = now_b;
        }

        if t_ms.is_multiple_of(HELLO_PERIOD_MS) {
            emit_hello(&Hello {
                t_s: t_ms / MILLIS_PER_SEC,
                ..hello
            });
            emit_git();
            emit_gpio(&GpioSample {
                boot: boot.is_high(),
                pmic_irq: pmic_irq.is_high(),
                tp: tp.is_high(),
                ioe: ioe.is_high(),
                busy: busy.is_high(),
            });
        }

        if t_ms.is_multiple_of(HEARTBEAT_PERIOD_MS) {
            emit_heartbeat(&Snapshot {
                t_s: t_ms / MILLIS_PER_SEC,
                btn_a: now_a,
                btn_b: now_b,
            });
        }

        t_ms = t_ms.saturating_add(POLL_PERIOD_MS);
        delay.delay_ms(POLL_PERIOD_MS);
    }
}

fn emit(line: &str) {
    print!("{line}\r\n");
}

fn emit_hello(hello: &Hello) {
    let mut buf = [0u8; HELLO_CAPACITY];
    if let Ok(line) = format_hello(hello, &mut buf) {
        emit(line);
    }
}

fn emit_git() {
    let mut buf = [0u8; GIT_CAPACITY];
    if let Ok(line) = format_git(
        env!("SIMPLE_DEBUG_GIT"),
        env!("SIMPLE_DEBUG_GIT_DIRTY") == "1",
        &mut buf,
    ) {
        emit(line);
    }
}

fn emit_gpio(sample: &GpioSample) {
    let mut buf = [0u8; GPIO_CAPACITY];
    if let Ok(line) = format_gpio(sample, &mut buf) {
        emit(line);
    }
}

fn emit_heartbeat(snapshot: &Snapshot) {
    let mut buf = [0u8; HEARTBEAT_CAPACITY];
    if let Ok(line) = format_heartbeat(snapshot, &mut buf) {
        emit(line);
    }
}

fn emit_edge(edge: &Edge) {
    let mut buf = [0u8; EDGE_CAPACITY];
    if let Ok(line) = format_edge(edge, &mut buf) {
        emit(line);
    }
}

fn reset_token(reason: Option<SocResetReason>) -> &'static str {
    match reason {
        Some(SocResetReason::ChipPowerOn) => "chip_power_on",
        Some(SocResetReason::CoreSw) => "core_sw",
        Some(SocResetReason::CoreDeepSleep) => "core_deep_sleep",
        Some(SocResetReason::CoreMwdt0) => "core_mwdt0",
        Some(SocResetReason::CoreMwdt1) => "core_mwdt1",
        Some(SocResetReason::CoreRtcWdt) => "core_rtc_wdt",
        Some(SocResetReason::CpuMwdt0) => "cpu_mwdt0",
        Some(SocResetReason::CpuSw) => "cpu_sw",
        Some(SocResetReason::CpuRtcWdt) => "cpu_rtc_wdt",
        Some(SocResetReason::SysBrownOut) => "sys_brown_out",
        Some(SocResetReason::SysRtcWdt) => "sys_rtc_wdt",
        Some(SocResetReason::CpuMwdt1) => "cpu_mwdt1",
        Some(SocResetReason::SysSuperWdt) => "sys_super_wdt",
        Some(SocResetReason::SysClkGlitch) => "sys_clk_glitch",
        Some(SocResetReason::CoreEfuseCrc) => "core_efuse_crc",
        Some(SocResetReason::CoreUsbUart) => "core_usb_uart",
        Some(SocResetReason::CoreUsbJtag) => "core_usb_jtag",
        Some(SocResetReason::CorePwrGlitch) => "core_pwr_glitch",
        None => "unknown",
    }
}
