//! PaperMono-Lite simple-debug tutorial firmware.
//!
//! # Purpose & Architecture
//! This firmware serves as an introductory, bare-metal proof-of-life reference
//! implementation for the M5Stack PaperMono-Lite (`C153-Lite`). It demonstrates
//! fundamental embedded Rust concepts described in *The Embedded Rust Book* and
//! *The Rust on ESP Book*:
//!
//! - **`#![no_std]` and `#![no_main]`**: Operates without the Rust standard library
//!   or an underlying operating system, taking direct control of hardware from the
//!   Espressif second-stage bootloader.
//! - **Blocking Hardware Abstraction Layer (`esp-hal`)**: Employs synchronous,
//!   blocking peripheral drivers rather than an asynchronous runtime or RTOS.
//! - **Deterministic Polling Loop**: Uses a periodic polling loop regulated by
//!   microsecond hardware delays (`esp_hal::delay::Delay`).
//! - **Zero Heap Allocation**: All telemetry buffers are fixed-size stack arrays
//!   (`[u8; N]`) formatted via [`papermono_log`], eliminating dynamic memory allocation
//!   and heap fragmentation risks.
//!
//! # Hardware Topology & Electrical Characteristics
//! - **USB-Serial/JTAG Telemetry**: Transmits line-oriented ASCII messages directly
//!   over the ESP32-S3 native USB-Serial/JTAG controller (`303a:1001`), requiring no
//!   external USB-to-UART bridge (such as a CH343).
//! - **Tactile User Buttons**:
//!   - `GPIO2`: BUTTON A (UP / `USER_KEY1`). Configured as active-low input with internal
//!     weak pull-up (`Pull::Up`). Pressing the switch grounds the pin.
//!   - `GPIO3`: BUTTON B (DOWN / `USER_KEY2`). Configured as active-low input with internal
//!     weak pull-up (`Pull::Up`). Note: GPIO3 is also an ESP32-S3 strapping pin (floating at
//!     reset). Pressing the switch grounds the pin.
//! - **Monitored Board Signal Nets**:
//!   - `GPIO0`: M5PM1 PMIC `BOOT_OUT`. Strapping pin (internal weak pull-up at reset).
//!   - `GPIO1`: M5PM1 PMIC IRQ (`G1_PY_IRQ`). Open-drain interrupt line.
//!   - `GPIO4`: FT6336G capacitive touch controller interrupt (`G4_TP_INT`). Active-low.
//!   - `GPIO7`: M5IOE1 I/O expander interrupt (`PYB_IRQ`).
//!   - `GPIO18`: SSD1677 e-paper controller `BUSY` signal. Active-high when refreshing.
//!
//!   All monitored board nets are sampled as high-impedance inputs without internal pulls
//!   (`Pull::None`) to prevent unwanted bias currents or interference with external circuits.
//!
//! # Safety & Hardware Conservation
//! To satisfy hardware safety constraints during early board bring-up:
//! - The SSD1677 e-paper panel is kept completely dormant; no SPI transactions or
//!   refresh waveforms are issued, avoiding uncalibrated LUT hazards.
//! - System I2C bus (`GPIO47`/`GPIO48`) and expander rails remain unclocked.
//! - PDM microphone clock/data pins (`GPIO45`/`GPIO46`) remain high-impedance.

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

// Binary image metadata for the ESP-IDF 2nd-stage bootloader.
// Enables image validation, partition table matching, and version tracking.
esp_bootloader_esp_idf::esp_app_desc!();

// Compile-time sanity verification that board crate pin constants match official schematics.
const _: () = {
    assert!(pins::BUTTON_A == 2);
    assert!(pins::BUTTON_B == 3);
    assert!(pins::PMIC_BOOT_OUT == 0);
    assert!(pins::PMIC_IRQ == 1);
    assert!(pins::TOUCH_INT == 4);
    assert!(pins::IOE1_IRQ == 7);
    assert!(pins::EPD_BUSY == 18);
};

/// Firmware entry point executed after the ESP-IDF second-stage bootloader transfers control.
///
/// # Hardware Initialization Sequence
/// 1. Initializes chip system clocks, power management, and peripheral singletons via
///    `esp_hal::init(Config::default())`.
/// 2. Configures hardware cycle-counter delay provider (`esp_hal::delay::Delay`).
/// 3. Instantiates `Input` pin drivers with appropriate electrical pull configurations:
///    - Pushbuttons (`GPIO2`, `GPIO3`): Configured with internal weak pull-ups (`Pull::Up`).
///    - Board status lines (`GPIO0`, `GPIO1`, `GPIO4`, `GPIO7`): Configured with
///      `Pull::None` to sample external logic states without back-powering uninitialized rails.
///    - E-paper BUSY signal (`GPIO18`): Configured with `Pull::Up` per SSD1677 datasheet and OTP-Demo.
/// 4. Queries SoC clock speeds and reset reason from the Real-Time Clock Controller (RTC_CNTL).
/// 5. Enters the non-terminating polling loop:
///    - 50 ms: Polls tactile button states; emits instantaneous transition telemetry on edge.
///    - 1000 ms: Emits periodic button status heartbeat (`heartbeat` line).
///    - 10000 ms: Emits board metadata (`hello`), git revision (`git`), and GPIO bus states (`gpio`).
#[main]
fn main() -> ! {
    // Take ownership of MCU peripherals and configure standard system clocks.
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    // Tactile user buttons: mechanical switches pull the net to ground when pressed.
    // We configure internal pull-ups so the default unpressed state reads high (logic 1).
    let btn_a = Input::new(
        peripherals.GPIO2,
        InputConfig::default().with_pull(Pull::Up),
    );
    let btn_b = Input::new(
        peripherals.GPIO3,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Board status and interrupt lines:
    // Left floating (`Pull::None`) because these lines either have dedicated external pull-ups
    // on the PCB or are driven by peripheral chips (PMIC, expander, touch digitizer).
    // Adding internal pull resistors here could cause leakage current into unpowered domains.
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
    // E-paper BUSY signal: SSD1677 datasheet and official OTP-Demo use pull-up.
    let busy = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Construct immutable identification record with boot-time telemetry.
    let hello = Hello {
        t_s: 0,
        image: IMAGE,
        sku: SKU,
        cpu_mhz: cpu_clock().as_mhz(),
        xtal_mhz: xtal_clock().as_mhz(),
        reset: reset_token(reset_reason()),
    };

    // Tracking state for button edge detection.
    let mut t_ms = 0_u32;
    let mut prev_a = btn_a.is_high();
    let mut prev_b = btn_b.is_high();

    // Deterministic polling loop.
    loop {
        let now_a = btn_a.is_high();
        let now_b = btn_b.is_high();

        // Edge detection: fire immediately on state transition.
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

        // Periodic 10-second banner: re-identifies firmware, build revision, and board rails.
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

        // Periodic 1-second heartbeat: confirms firmware liveness and current button states.
        if t_ms.is_multiple_of(HEARTBEAT_PERIOD_MS) {
            emit_heartbeat(&Snapshot {
                t_s: t_ms / MILLIS_PER_SEC,
                btn_a: now_a,
                btn_b: now_b,
            });
        }

        // Advance simulated time tick and delay for poll interval (50 ms).
        t_ms = t_ms.saturating_add(POLL_PERIOD_MS);
        delay.delay_ms(POLL_PERIOD_MS);
    }
}

/// Transmits a formatted line with a CRLF terminator over native USB-Serial/JTAG.
///
/// Uses `esp_println::print!` which routes directly to the hardware FIFO of the
/// ESP32-S3 USB Serial/JTAG controller without requiring UART interrupts or DMA.
fn emit(line: &str) {
    print!("{line}\r\n");
}

/// Formats and emits a device identification message (`Hello`).
///
/// Allocates a dedicated stack buffer of size [`HELLO_CAPACITY`] to avoid heap allocation.
fn emit_hello(hello: &Hello) {
    let mut buf = [0u8; HELLO_CAPACITY];
    if let Ok(line) = format_hello(hello, &mut buf) {
        emit(line);
    }
}

/// Formats and emits build git commit metadata embedded at compile-time.
///
/// The commit hash and dirty status are captured by the workspace build environment.
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

/// Formats and emits the current logic levels of monitored board nets (`GpioSample`).
fn emit_gpio(sample: &GpioSample) {
    let mut buf = [0u8; GPIO_CAPACITY];
    if let Ok(line) = format_gpio(sample, &mut buf) {
        emit(line);
    }
}

/// Formats and emits the periodic 1 Hz button liveness heartbeat (`Snapshot`).
fn emit_heartbeat(snapshot: &Snapshot) {
    let mut buf = [0u8; HEARTBEAT_CAPACITY];
    if let Ok(line) = format_heartbeat(snapshot, &mut buf) {
        emit(line);
    }
}

/// Formats and emits an edge transition event triggered by tactile button presses or releases.
fn emit_edge(edge: &Edge) {
    let mut buf = [0u8; EDGE_CAPACITY];
    if let Ok(line) = format_edge(edge, &mut buf) {
        emit(line);
    }
}

/// Translates the hardware reset reason reported by the RTC controller into a static string token.
///
/// Maps Espressif `SocResetReason` hardware registers into parser-friendly tokens recognized
/// by `papermono-log` and host monitoring tools.
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
