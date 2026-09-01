//! PaperMono-Lite embassy-debug image.
//!
//! Embassy tasks print the same CDC grammar as Path A
//! (`simple-debug:` prefix), with
//! `hello image=embassy-debug`. Optional stages: I2C/touch, PDM mic,
//! OTP five-card walk (splash / shapes / legend / tones /
//! targets). Tones are 4-gray; splash / shapes / legend
//! are mono. After six partials, one mono full. No NFC,
//! no LoRa, no LUT, no GPIO45/46 latch. Optional `--features radio`
//! prints `wifi n=` / `ble n=` only.

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

esp_bootloader_esp_idf::esp_app_desc!();

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

#[esp_hal::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    #[cfg(feature = "radio")]
    {
        esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
        esp_alloc::heap_allocator!(size: 64 * 1024);
    }
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let btn_a = Input::new(
        peripherals.GPIO2,
        InputConfig::default().with_pull(Pull::Up),
    );
    let btn_b = Input::new(
        peripherals.GPIO3,
        InputConfig::default().with_pull(Pull::Up),
    );
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
    // OTP-Demo `edp_spi::begin` pull-up on BUSY.
    let busy = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(Pull::Up),
    );
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

    #[cfg(feature = "radio")]
    {
        // BLE first so coexistence is up. Separate tasks so a stuck
        // Wi-Fi scan does not starve BLE listen.
        spawner.spawn(radio::ble_run(peripherals.BT).unwrap());
        spawner.spawn(radio::wifi_run(peripherals.WIFI).unwrap());
    }

    let hello = Hello {
        t_s: 0,
        image: IMAGE_EMBASSY,
        sku: SKU,
        cpu_mhz: cpu_clock().as_mhz(),
        xtal_mhz: xtal_clock().as_mhz(),
        reset: reset_token(reset_reason()),
    };

    // GPIO42 / LEDC stay untouched (`beep` is parked). Taking
    // JTAG `MTMS` for the buzzer coincided with wedged cards.

    #[cfg(feature = "touch")]
    {
        use esp_hal::i2c::master::{Config, I2c};

        // `Config::default()` is 100 kHz (esp-hal I2C).
        let mut i2c = I2c::new(peripherals.I2C0, Config::default())
            .expect("I2C0")
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48);
        let _ = touch_bus::bring_up(&mut i2c).await;
        #[cfg(feature = "sleep")]
        crate::sleep_wake::maybe_rtc_10s(&mut i2c).await;

        // PDM stays up. Hold A ~1 s dumps PCM when the card UI is live.
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
            share::UI_LIVE.store(true, core::sync::atomic::Ordering::Relaxed);
            share::BTN_A.store(btn_a.is_high(), core::sync::atomic::Ordering::Relaxed);
            share::BTN_B.store(btn_b.is_high(), core::sync::atomic::Ordering::Relaxed);
            share::TP.store(tp.is_high(), core::sync::atomic::Ordering::Relaxed);
            share::BUSY.store(busy.is_high(), core::sync::atomic::Ordering::Relaxed);
            spawner.spawn(ui::run(i2c, panel, btn_a, btn_b, tp, busy).unwrap());
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
            loop {
                Timer::after(Duration::from_secs(60)).await;
            }
        }
    }

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
