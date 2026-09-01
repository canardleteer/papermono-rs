//! USB-Serial/JTAG line emit. Firmware owns pins; crate `papermono-log`
//! owns the strings.

use esp_println::print;
use papermono_log::{
    format_edge, format_git, format_gpio, format_heartbeat, format_hello, Edge, GpioSample, Hello,
    Snapshot, EDGE_CAPACITY, GIT_CAPACITY, GPIO_CAPACITY, HEARTBEAT_CAPACITY, HELLO_CAPACITY,
};
#[cfg(feature = "touch")]
use papermono_log::{format_lamp, LAMP_CAPACITY};
#[cfg(feature = "panel")]
use papermono_log::{format_scene, Scene, SCENE_CAPACITY};

pub fn emit(line: &str) {
    print!("{line}\r\n");
}

pub fn hello(hello: &Hello) {
    let mut buf = [0u8; HELLO_CAPACITY];
    if let Ok(line) = format_hello(hello, &mut buf) {
        emit(line);
    }
}

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

pub fn gpio(sample: &GpioSample) {
    let mut buf = [0u8; GPIO_CAPACITY];
    if let Ok(line) = format_gpio(sample, &mut buf) {
        emit(line);
    }
}

pub fn heartbeat(snapshot: &Snapshot) {
    let mut buf = [0u8; HEARTBEAT_CAPACITY];
    if let Ok(line) = format_heartbeat(snapshot, &mut buf) {
        emit(line);
    }
}

pub fn edge(edge: &Edge) {
    let mut buf = [0u8; EDGE_CAPACITY];
    if let Ok(line) = format_edge(edge, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "touch")]
pub fn i2c(sample: &papermono_log::I2cSample) {
    let mut buf = [0u8; papermono_log::I2C_CAPACITY];
    if let Ok(line) = papermono_log::format_i2c(sample, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "touch")]
pub fn touch(sample: &papermono_log::TouchSample) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch(sample, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "panel")]
pub fn touch_target(id: u8, kind: &str, x: u16, y: u16, r: u16) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch_target(id, kind, x, y, r, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "panel")]
pub fn touch_at(id: u8, sample: &papermono_log::TouchSample, tx: u16, ty: u16) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch_at(id, sample, tx, ty, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "panel")]
pub fn touch_verdict(id: u8, verdict: &str, x: u16, y: u16, tx: u16, ty: u16, d: u16) {
    let mut buf = [0u8; papermono_log::TOUCH_CAPACITY];
    if let Ok(line) = papermono_log::format_touch_verdict(id, verdict, x, y, tx, ty, d, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "mic")]
pub fn mic(sample: &papermono_log::MicSample) {
    let mut buf = [0u8; papermono_log::MIC_CAPACITY];
    if let Ok(line) = papermono_log::format_mic(sample, &mut buf) {
        emit(line);
    }
}

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

#[cfg(feature = "panel")]
pub fn scene(scene: Scene) {
    let mut buf = [0u8; SCENE_CAPACITY];
    if let Ok(line) = format_scene(scene, &mut buf) {
        emit(line);
    }
}

pub fn leftover(sample: &papermono_log::LeftoverSample) {
    let mut buf = [0u8; papermono_log::LEFTOVER_CAPACITY];
    if let Ok(line) = papermono_log::format_leftover(sample, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "sleep")]
pub fn sleep_rtc(secs: u8) {
    let mut buf = [0u8; papermono_log::SLEEP_CAPACITY];
    if let Ok(line) = papermono_log::format_sleep_rtc(secs, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "sleep")]
pub fn sleep_abort() {
    let mut buf = [0u8; papermono_log::SLEEP_CAPACITY];
    if let Ok(line) = papermono_log::format_sleep_abort(&mut buf) {
        emit(line);
    }
}

#[cfg(feature = "sleep")]
pub fn wake(src: u8) {
    let mut buf = [0u8; papermono_log::WAKE_CAPACITY];
    if let Ok(line) = papermono_log::format_wake(src, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "touch")]
pub fn charge(sample: &papermono_log::ChargeSample) {
    let mut buf = [0u8; papermono_log::CHARGE_CAPACITY];
    if let Ok(line) = papermono_log::format_charge(sample, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "radio")]
pub fn wifi(n: u16) {
    let mut buf = [0u8; papermono_log::WIFI_CAPACITY];
    if let Ok(line) = papermono_log::format_wifi(n, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "radio")]
pub fn ble(n: u16) {
    let mut buf = [0u8; papermono_log::BLE_CAPACITY];
    if let Ok(line) = papermono_log::format_ble(n, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "touch")]
pub fn lamp(duty: u16) {
    let mut buf = [0u8; LAMP_CAPACITY];
    if let Ok(line) = format_lamp(duty, &mut buf) {
        emit(line);
    }
}

#[cfg(feature = "panel")]
pub fn panel(stamp: &papermono_log::PanelStamp) {
    let mut buf = [0u8; papermono_log::PANEL_CAPACITY];
    if let Ok(line) = papermono_log::format_panel(stamp, &mut buf) {
        emit(line);
    }
}
