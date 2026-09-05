//! Passive magnetic buzzer driver on GPIO42 (`BB_PWM`) via ESP32-S3 LEDC.
//!
//! # Hardware Topology & Electrical Constraints
//! - **Pin Multiplexing**: The passive magnetic buzzer connects to `GPIO42` (PinMap `BB_PWM`),
//!   a multifunctional pad shared with JTAG `MTMS` (ESP32-S3 datasheet Table 2-4).
//! - **Output Matrix Routing**: The ESP32-S3 GPIO matrix multiplexes the LEDC low-speed
//!   peripheral output onto `GPIO42` with push-pull drive mode, safely disconnecting the pad
//!   from JTAG `MTMS` without disturbing USB-Serial/JTAG CDC communication.
//! - **Modulation Architecture**: Driven via LEDC Low-Speed Timer 3 and Channel 7
//!   (matching the factory demo in [M5PaperMono-UserDemo](https://github.com/m5stack/M5PaperMono-UserDemo)
//!   `hal_buzzer.cpp`).
//! - **Acoustic Amplitude & Volume Scaling**: For a passive electromagnetic transducer driven
//!   by a periodic square wave, acoustic volume is modulated by scaling the duty cycle:
//!   - 25% duty cycle ([`buzzer::DUTY_MAX_PCT`]) delivers maximum acoustic power on hardware.
//!   - 0% duty cycle holds the pin at constant DC low, silencing the transducer.
//!   - Volume levels (0..=100%) map linearly to 0..=25% duty via [`buzzer::volume_to_duty_pct`].
//! - **Asynchronous Cooperative Execution**: An Embassy background worker task coordinates
//!   tone requests received over an in-memory queue ([`Channel`]), ensuring UI rendering and
//!   touch digitizer polling loops remain strictly non-blocking.

use core::sync::atomic::{AtomicU8, Ordering};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::DriveMode;
use esp_hal::ledc::{
    channel::{self, ChannelIFace},
    timer::{self, TimerIFace},
    LSGlobalClkSource, Ledc, LowSpeed,
};
use esp_hal::time::Rate;
use m5stack_papermono_lite::buzzer;

/// Default initial buzzer volume level upon boot (50%).
pub const DEFAULT_VOLUME: u8 = 50;

/// Tick chirp duration in milliseconds (10 ms).
#[allow(dead_code)]
pub const TICK_MS: u32 = 10;

/// Internal tone descriptor queued for asynchronous background generation.
#[derive(Clone, Copy)]
pub struct BeepRequest {
    /// Tone duration in milliseconds.
    pub ms: u32,
}

/// Static multi-producer single-consumer channel queue for pending buzzer tone commands.
static BEEP_CHANNEL: Channel<CriticalSectionRawMutex, BeepRequest, 4> = Channel::new();

/// Atomic storage holding the current global buzzer volume level (0..=100%).
static VOLUME: AtomicU8 = AtomicU8::new(DEFAULT_VOLUME);

/// Adjusts the global buzzer volume level percentage (clamped to 0..=100%).
#[allow(dead_code)]
pub fn set_volume(volume: u8) {
    VOLUME.store(volume.min(100), Ordering::Relaxed);
}

/// Reads the current global buzzer volume level percentage (0..=100%).
#[allow(dead_code)]
#[must_use]
pub fn volume() -> u8 {
    VOLUME.load(Ordering::Relaxed)
}

/// Queues a tone request of the specified duration for non-blocking playback by the buzzer task.
///
/// If the request channel is full, the request is discarded to preserve system responsiveness.
#[allow(dead_code)]
pub fn tone(ms: u32) {
    let _ = BEEP_CHANNEL.try_send(BeepRequest { ms });
}

/// Dispatches the standard tactile key-click feedback sound (2 kHz for 40 ms).
#[allow(dead_code)]
pub fn click() {
    tone(buzzer::BEEP_MS as u32);
}

/// Dispatches a brief tick chirp for tactile volume slider dragging (2 kHz for 10 ms).
#[allow(dead_code)]
pub fn tick() {
    tone(TICK_MS);
}

/// Retained no-op placeholder function for backwards compatibility with earlier bring-up tests.
#[allow(dead_code)]
pub fn ask() {}

/// Cooperative asynchronous background task driving LEDC hardware PWM for passive buzzer output.
///
/// # Hardware Peripherals Owned
/// - `LEDC`: Configures global low-speed clock to APBClk and allocates Timer 3.
/// - `GPIO42`: Connected via the GPIO matrix to Channel 7 in push-pull drive mode.
#[embassy_executor::task]
pub async fn run(
    ledc_peripheral: esp_hal::peripherals::LEDC<'static>,
    buzzer_pin: esp_hal::peripherals::GPIO42<'static>,
) {
    let mut ledc = Ledc::new(ledc_peripheral);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer = ledc.timer::<LowSpeed>(timer::Number::Timer3);
    let _ = timer.configure(timer::config::Config {
        duty: timer::config::Duty::Duty10Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: Rate::from_hz(buzzer::BEEP_HZ),
    });

    let mut channel = ledc.channel(channel::Number::Channel7, buzzer_pin);
    let _ = channel.configure(channel::config::Config {
        timer: &timer,
        duty_pct: 0,
        drive_mode: DriveMode::PushPull,
    });

    loop {
        let req = BEEP_CHANNEL.receive().await;
        let vol = volume();
        let duty = buzzer::volume_to_duty_pct(vol);
        if duty > 0 && req.ms > 0 {
            let _ = channel.set_duty(duty);
            Timer::after(Duration::from_millis(req.ms.into())).await;
            let _ = channel.set_duty(0);
        }
    }
}
