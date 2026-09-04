//! Asynchronous USB-Serial/JTAG line-oriented telemetry emitter.
//!
//! # Architecture & Protocol Separation
//! This module coordinates transmission of ASCII telemetry lines over the
//! ESP32-S3 native USB-Serial/JTAG peripheral FIFO (`esp_println::print!`).
//!
//! To preserve separation of concerns and maintain zero-allocation guarantees:
//! - **Firmware Ownership**: This module owns the hardware execution context,
//!   per-call fixed stack buffers (`[u8; N]`), and serial print invocation.
//! - **Crate Ownership**: The [`papermono_log`] crate owns message schemas,
//!   formatting logic, maximum byte capacities, and grammar compliance.
//! - **Zero Dynamic Allocation**: No formatting operation invokes the global
//!   allocator; lines are formatted directly into fixed-size stack arrays.

use esp_println::print;
use papermono_log::{
    format_edge, format_git, format_gpio, format_heartbeat, format_hello, Edge, GpioSample, Hello,
    Snapshot, EDGE_CAPACITY, GIT_CAPACITY, GPIO_CAPACITY, HEARTBEAT_CAPACITY, HELLO_CAPACITY,
};
#[cfg(feature = "touch")]
use papermono_log::{format_lamp, LAMP_CAPACITY};
#[cfg(feature = "panel")]
use papermono_log::{format_scene, format_snowflake, Scene, SCENE_CAPACITY, SNOWFLAKE_CAPACITY};

/// Writes a raw string slice followed by CRLF (`\r\n`) to the native USB FIFO.
pub fn emit(line: &str) {
    print!("{line}\r\n");
}

/// Emits the periodic or boot device identification banner (`Hello`).
pub fn hello(hello: &Hello) {
    let mut buf = [0u8; HELLO_CAPACITY];
    if let Ok(line) = format_hello(hello, &mut buf) {
        emit(line);
    }
}

/// Emits compile-time git version and repository clean/dirty status.
pub fn git() {
    let mut buf = [0u8; GIT_CAPACITY];
    if let Ok(line) = format_git(
        env!("EMBASSY_DEBUG_GIT"),
        env!("EMBASSY_DEBUG_GIT_DIRTY") == "1",
        &mut buf,
    ) {
        emit(line);
    }
}

/// Emits instantaneous logic levels of monitored board nets.
pub fn gpio(sample: &GpioSample) {
    let mut buf = [0u8; GPIO_CAPACITY];
    if let Ok(line) = format_gpio(sample, &mut buf) {
        emit(line);
    }
}

/// Emits periodic 1 Hz liveness heartbeat and tactile button states.
pub fn heartbeat(snapshot: &Snapshot) {
    let mut buf = [0u8; HEARTBEAT_CAPACITY];
    if let Ok(line) = format_heartbeat(snapshot, &mut buf) {
        emit(line);
    }
}

/// Emits instantaneous button press or release transition events.
pub fn edge(edge: &Edge) {
    let mut buf = [0u8; EDGE_CAPACITY];
    if let Ok(line) = format_edge(edge, &mut buf) {
        emit(line);
    }
}

/// Emits system I2C bus detection roster (`I2cSample`).
#[cfg(feature = "touch")]
pub fn i2c(sample: &papermono_log::I2cSample) {
    let mut buf = [0u8; papermono_log::I2C_CAPACITY];
    if let Ok(line) = papermono_log::format_i2c(sample, &mut buf) {
        emit(line);
    }
}

/// Emits touch coordinate readings from the FT6336G capacitive digitizer.
#[cfg(feature = "touch")]
pub fn touch(sample: &papermono_log::TouchSample) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch(sample, &mut buf) {
        emit(line);
    }
}

/// Emits calibration target rendering announcements during the touch walk.
#[cfg(feature = "panel")]
pub fn touch_target(id: u8, kind: &str, x: u16, y: u16, r: u16) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch_target(id, kind, x, y, r, &mut buf) {
        emit(line);
    }
}

/// Emits touch samples registered while targeting a specific calibration point.
#[cfg(feature = "panel")]
pub fn touch_at(id: u8, sample: &papermono_log::TouchSample, tx: u16, ty: u16) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch_at(id, sample, tx, ty, &mut buf) {
        emit(line);
    }
}

/// Emits final hit/miss/abort verdict and Euclidean error for a calibration target.
#[cfg(feature = "panel")]
pub fn touch_verdict(id: u8, verdict: &str, x: u16, y: u16, tx: u16, ty: u16, d: u16) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch_verdict(id, verdict, x, y, tx, ty, d, &mut buf) {
        emit(line);
    }
}

/// Emits periodic PDM microphone energy metrics (RMS and peak values).
#[cfg(feature = "mic")]
pub fn mic(sample: &papermono_log::MicSample) {
    let mut buf = [0u8; papermono_log::MIC_CAPACITY];
    if let Ok(line) = papermono_log::format_mic(sample, &mut buf) {
        emit(line);
    }
}

/// Emits streaming audio sample dumps formatted in rows of fixed width.
#[cfg(feature = "mic")]
pub fn mic_pcm(hz: u32, samples: &[i16]) {
    let mut header = [0u8; papermono_log::PCM_HEADER_CAPACITY];
    if let Ok(line) = papermono_log::format_mic_pcm_header(hz, samples.len(), &mut header) {
        emit(line);
    }
    let mut offset = 0;
    while offset < samples.len() {
        let end = core::cmp::min(offset + papermono_log::PCM_ROW_SAMPLES, samples.len());
        let mut row = [0u8; papermono_log::PCM_ROW_CAPACITY];
        if let Ok(line) = papermono_log::format_mic_pcm_row(offset, &samples[offset..end], &mut row)
        {
            emit(line);
        }
        offset = end;
    }
}

/// Emits interactive UI card transition events (`Scene`).
#[cfg(feature = "panel")]
pub fn scene(scene: Scene) {
    let mut buf = [0u8; SCENE_CAPACITY];
    if let Ok(line) = format_scene(scene, &mut buf) {
        emit(line);
    }
}

/// Emits procedural snowflake render duration in microseconds.
#[cfg(feature = "panel")]
pub fn snowflake(us: u32) {
    let mut buf = [0u8; SNOWFLAKE_CAPACITY];
    if let Ok(line) = format_snowflake(us, &mut buf) {
        emit(line);
    }
}

/// Emits logic levels of unassigned radio pins on Lite hardware.
pub fn leftover(sample: &papermono_log::LeftoverSample) {
    let mut buf = [0u8; papermono_log::LEFTOVER_CAPACITY];
    if let Ok(line) = papermono_log::format_leftover(sample, &mut buf) {
        emit(line);
    }
}

/// Emits battery charging status and IP2315 registers.
#[cfg(feature = "touch")]
pub fn charge(sample: &papermono_log::ChargeSample) {
    let mut buf = [0u8; papermono_log::CHARGE_CAPACITY];
    if let Ok(line) = papermono_log::format_charge(sample, &mut buf) {
        emit(line);
    }
}

/// Emits count of unique Wi-Fi access points observed during passive scan.
#[cfg(feature = "radio")]
pub fn wifi(n: u16) {
    let mut buf = [0u8; papermono_log::WIFI_CAPACITY];
    if let Ok(line) = papermono_log::format_wifi(n, &mut buf) {
        emit(line);
    }
}

/// Emits count of BLE advertisement packets received during passive scan.
#[cfg(feature = "radio")]
pub fn ble(n: u16) {
    let mut buf = [0u8; papermono_log::BLE_CAPACITY];
    if let Ok(line) = papermono_log::format_ble(n, &mut buf) {
        emit(line);
    }
}

/// Emits frontlight LED PWM duty cycle level.
#[cfg(feature = "touch")]
pub fn lamp(duty: u16) {
    let mut buf = [0u8; LAMP_CAPACITY];
    if let Ok(line) = format_lamp(duty, &mut buf) {
        emit(line);
    }
}

/// Emits display refresh completion telemetry (`PanelStamp`).
#[cfg(feature = "panel")]
pub fn panel(stamp: &papermono_log::PanelStamp) {
    let mut buf = [0u8; papermono_log::PANEL_CAPACITY];
    if let Ok(line) = papermono_log::format_panel(stamp, &mut buf) {
        emit(line);
    }
}

/// Emits BLE pairing 6-digit numeric passkey.
#[cfg(feature = "radio")]
pub fn pair_pin(pin: u32) {
    let mut buf = [0u8; papermono_log::PAIR_CAPACITY];
    if let Ok(line) = papermono_log::format_pair_pin(pin, &mut buf) {
        emit(line);
    }
}

/// Emits BLE pairing successful completion event.
#[cfg(feature = "radio")]
pub fn pair_ok() {
    let mut buf = [0u8; papermono_log::PAIR_CAPACITY];
    if let Ok(line) = papermono_log::format_pair_ok(&mut buf) {
        emit(line);
    }
}

/// Emits BLE pairing failure reason event.
#[cfg(feature = "radio")]
pub fn pair_fail(why: &str) {
    let mut buf = [0u8; papermono_log::PAIR_CAPACITY];
    if let Ok(line) = papermono_log::format_pair_fail(why, &mut buf) {
        emit(line);
    }
}

/// Emits BLE peripheral state transition event.
#[cfg(feature = "radio")]
pub fn pair_state(state: &str) {
    let mut buf = [0u8; papermono_log::PAIR_CAPACITY];
    if let Ok(line) = papermono_log::format_pair_state(state, &mut buf) {
        emit(line);
    }
}
