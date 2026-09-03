//! PaperMono-Lite Embassy interactive debug tutorial firmware.
//!
//! # Architecture & Purpose
//! This firmware serves as an advanced, asynchronous reference application for
//! the M5Stack PaperMono-Lite (`C153-Lite`), demonstrating principles from
//! *The Embassy Book*, *The Rust on ESP Book*, and *The Embedded Rust Book*:
//!
//! - **Asynchronous Cooperative Multitasking**: Built on the `embassy-executor`
//!   runtime and `esp-rtos`, enabling concurrent execution of CDC telemetry,
//!   touch digitizer polling, display rendering, and audio sampling without
//!   traditional thread preemption or OS overhead.
//! - **Hardware Abstraction Layer**: Leverages `esp-hal` in asynchronous mode
//!   for non-blocking timers, I2C, SPI, and PDM/I2S DMA transfers.
//! - **Line-Oriented Telemetry Protocol**: Shares the identical telemetry grammar
//!   with `simple-debug` (`simple-debug:` line prefix, parsed by `papermono-log`),
//!   identifying itself via `image=embassy-debug`.
//! - **Interactive Five-Card Demonstration**: Implements a five-card user interface:
//!   1. `Splash`: High-contrast rendering of the Rust mascot Ferris.
//!   2. `Shapes`: Geometric calibration patterns (boxes, circles, diagonal crossbars).
//!   3. `Legend`: On-device quick-reference card for keys, touch gestures, and rails.
//!   4. `Tones`: 4-gray quadrant test validating SSD1677 OTP grayscale waveforms.
//!   5. `Targets`: Interactive touch accuracy calibration and latency verification.
//! - **Hardware Display Protection Contract**: Enforces strict OTP waveform safety:
//!   after a fixed budget of partial refreshes ([`panel::PARTIALS_BEFORE_FULL`]),
//!   a full refresh is mandated to eliminate DC bias accumulation and prevent
//!   permanent panel ghosting.
//!
//! # Memory & Execution Model
//! - **Static Allocation & Zero Heap**: Default builds require no dynamic heap
//!   allocator; tasks, peripheral drivers, and large display planes are allocated
//!   statically using [`static_cell::ConstStaticCell`].
//! - **Optional Features**:
//!   - `touch` (default): System I2C bus, M5IOE1 expander, M5PM1 PMIC, FT6336G digitizer.
//!   - `panel` (default): SSD1677 OTP e-paper display with 5-card interactive UI.
//!   - `mic` (opt-in): PDM microphone RMS and peak energy measurement via DMA.
//!   - `radio` (opt-in): Wi-Fi and BLE passive scanning (requires `esp-alloc` heap).
//!   - `sleep` (opt-in): RX8130CE RTC wake timer and PM1 deep shutdown sequencing.

#![no_std]
#![no_main]

#[allow(dead_code)]
mod beep;
mod cdc;
#[cfg(feature = "panel")]
mod draw;
mod heartbeat;
#[cfg(feature = "touch")]
mod ioe;
#[cfg(feature = "mic")]
mod mic;
#[cfg(feature = "panel")]
mod panel;
#[cfg(feature = "radio")]
mod radio;
#[cfg(feature = "panel")]
mod share;
#[cfg(feature = "sleep")]
mod sleep_wake;
#[cfg(feature = "panel")]
mod targets;
#[cfg(feature = "touch")]
mod touch_bus;
#[cfg(feature = "panel")]
mod ui;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
#[cfg(feature = "radio")]
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::clock::{cpu_clock, xtal_clock};
use esp_hal::gpio::{Input, InputConfig, Pull};
#[cfg(feature = "radio")]
use esp_hal::ram;
use esp_hal::rtc_cntl::SocResetReason;
use esp_hal::system::reset_reason;
use esp_hal::timer::timg::TimerGroup;
use m5stack_papermono_lite::pins;
use m5stack_papermono_lite::SKU;
use papermono_log::{Hello, IMAGE_EMBASSY};

// Application descriptor for the ESP-IDF second-stage bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

// Compile-time pinout verification against the official M5Stack PaperMono schematic.
const _: () = {
    assert!(pins::BUTTON_A == 2);
    assert!(pins::BUTTON_B == 3);
    assert!(pins::PMIC_BOOT_OUT == 0);
    assert!(pins::PMIC_IRQ == 1);
    assert!(pins::TOUCH_INT == 4);
    assert!(pins::LEFTOVER_LORA_IRQ == 5);
    assert!(pins::LEFTOVER_NFC_IRQ == 6);
    assert!(pins::IOE1_IRQ == 7);
    assert!(pins::EPD_BUSY == 18);
    assert!(pins::LEFTOVER_SX_BUSY == 21);
    assert!(pins::PDM_CLK == 45);
    assert!(pins::PDM_DAT == 46);
    assert!(pins::SYS_I2C_SDA == 47);
    assert!(pins::SYS_I2C_SCL == 48);
    assert!(pins::BUZZER == 42);
};

/// Main asynchronous entry point invoked by the Embassy executor.
///
/// # System Initialization Flow
/// 1. Clocks & Peripherals: Initializes core MCU clocks and peripheral singletons via
///    `esp_hal::init`.
/// 2. RTOS / Timer Integration: Configures `TIMG0.timer0` as the hardware time source
///    for the Embassy async runtime scheduler (`esp_rtos::start`).
/// 3. Digital Input Configuration:
///    - Buttons (`GPIO2`, `GPIO3`): Configured with internal pull-ups (`Pull::Up`).
///    - Status & Interrupts (`GPIO0`, `GPIO1`, `GPIO4`, `GPIO7`, `GPIO18`): Configured
///      floating (`Pull::None`) to prevent back-powering sleeping peripherals.
///    - Radio leftovers (`GPIO5`, `GPIO6`, `GPIO21`): Configured as high-impedance inputs.
/// 4. Peripheral Bus Bring-Up:
///    - I2C Bus (`I2C0` on `GPIO47`/`GPIO48`): Brought up at 100 kHz.
///    - Power Rails: Raises PMIC outputs and M5IOE1 expander gates (`EPD_VDD`, `TOUCH_VDD`).
/// 5. Task Spawning:
///    - Spawns interactive UI task (`ui::run`) and asynchronous telemetry worker (`heartbeat::run`).
#[esp_hal::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize MCU clock distribution and power controllers.
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // When wireless radio scanning is active, initialize heap allocators required by esp-radio.
    #[cfg(feature = "radio")]
    {
        esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
        esp_alloc::heap_allocator!(size: 64 * 1024);
    }

    // Hand over hardware TimerGroup 0 to esp-rtos to drive the Embassy time driver.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    // Tactile user pushbuttons with internal pull-ups (active-low switches).
    let btn_a = Input::new(
        peripherals.GPIO2,
        InputConfig::default().with_pull(Pull::Up),
    );
    let btn_b = Input::new(
        peripherals.GPIO3,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Peripheral interrupt lines: monitored as high-impedance inputs without pulls.
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
    let ioe_irq = Input::new(
        peripherals.GPIO7,
        InputConfig::default().with_pull(Pull::None),
    );
    // E-paper BUSY signal: SSD1677 datasheet and official OTP-Demo use pull-up.
    let busy = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Unassigned radio pins on PaperMono-Lite: safely left as undriven inputs.
    let lora_irq = Input::new(
        peripherals.GPIO5,
        InputConfig::default().with_pull(Pull::None),
    );
    let nfc_irq = Input::new(
        peripherals.GPIO6,
        InputConfig::default().with_pull(Pull::None),
    );
    let sx_busy = Input::new(
        peripherals.GPIO21,
        InputConfig::default().with_pull(Pull::None),
    );

    // Optional wireless scanning tasks: BLE must start before Wi-Fi for coexistence.
    #[cfg(feature = "radio")]
    {
        spawner.spawn(radio::ble_run(peripherals.BT).unwrap());
        spawner.spawn(radio::wifi_run(peripherals.WIFI).unwrap());
    }

    // Capture boot identification metadata for telemetry.
    let hello = Hello {
        t_s: 0,
        image: IMAGE_EMBASSY,
        sku: SKU,
        cpu_mhz: cpu_clock().as_mhz(),
        xtal_mhz: xtal_clock().as_mhz(),
        reset: reset_token(reset_reason()),
    };

    // System I2C bus and peripheral initialization (touch, expander, PMIC, display).
    #[cfg(feature = "touch")]
    {
        use esp_hal::i2c::master::{Config, I2c};

        // Initialize I2C0 master on GPIO47 (SDA) and GPIO48 (SCL) at 100 kHz.
        let mut i2c = I2c::new(peripherals.I2C0, Config::default())
            .expect("I2C0")
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48);

        // Bring up system expander rails and configure touch digitizer.
        let _ = touch_bus::bring_up(&mut i2c).await;

        // If sleep feature is enabled, evaluate RTC wake status.
        #[cfg(feature = "sleep")]
        crate::sleep_wake::maybe_rtc_10s(&mut i2c).await;

        // Optional PDM microphone sampling task.
        #[cfg(feature = "mic")]
        {
            use m5stack_papermono_lite::ioe1;

            let _ = ioe::set_push_pull_output(&mut i2c, ioe1::PDM_VDD_ENABLE, true);
            Timer::after(Duration::from_millis(20)).await;
            spawner.spawn(
                mic::run(
                    peripherals.I2S0,
                    peripherals.DMA_CH0,
                    peripherals.GPIO45.into(),
                    peripherals.GPIO46.into(),
                )
                .unwrap(),
            );
        }

        // Initialize SSD1677 OTP e-paper panel driver over SPI2.
        #[cfg(feature = "panel")]
        if let Some(panel) = panel::begin(
            &mut i2c,
            peripherals.SPI2,
            peripherals.GPIO14.into(),
            peripherals.GPIO15.into(),
            peripherals.GPIO16.into(),
            peripherals.GPIO17.into(),
            &busy,
        )
        .await
        {
            // Publish initial button and bus states into the global atomic ledger.
            share::UI_LIVE.store(true, core::sync::atomic::Ordering::Relaxed);
            share::BTN_A.store(btn_a.is_high(), core::sync::atomic::Ordering::Relaxed);
            share::BTN_B.store(btn_b.is_high(), core::sync::atomic::Ordering::Relaxed);
            share::TP.store(tp.is_high(), core::sync::atomic::Ordering::Relaxed);
            share::BUSY.store(busy.is_high(), core::sync::atomic::Ordering::Relaxed);

            // Spawn the interactive UI card navigator task.
            spawner.spawn(ui::run(i2c, panel, btn_a, btn_b, tp, busy).unwrap());

            // Spawn the background heartbeat and telemetry reporter task.
            spawner.spawn(
                heartbeat::run(
                    heartbeat::Inputs {
                        btn_a: None,
                        btn_b: None,
                        boot,
                        pmic_irq,
                        tp: None,
                        ioe: ioe_irq,
                        busy: None,
                        lora_irq,
                        nfc_irq,
                        sx_busy,
                    },
                    hello,
                )
                .unwrap(),
            );

            // Main task suspends while Embassy workers execute concurrently.
            loop {
                Timer::after(Duration::from_secs(60)).await;
            }
        }
    }

    // Fallback heartbeat task if panel feature is disabled.
    spawner.spawn(
        heartbeat::run(
            heartbeat::Inputs {
                btn_a: Some(btn_a),
                btn_b: Some(btn_b),
                boot,
                pmic_irq,
                tp: Some(tp),
                ioe: ioe_irq,
                busy: Some(busy),
                lora_irq,
                nfc_irq,
                sx_busy,
            },
            hello,
        )
        .unwrap(),
    );

    loop {
        Timer::after(Duration::from_secs(60)).await;
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
