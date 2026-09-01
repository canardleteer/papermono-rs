//! Stage C: self-hosted PDM RMS/peak on USB-Serial/JTAG. No cloud.
//!
//! Known-tone dump matches sticky `--features mic`: BUTTON A prints
//! two 16 kHz windows with `hz=0` (board plays nothing). Period is
//! counted in the rows. 16 kHz / right (UserDemo). Caller enables
//! `PYG12` before spawn.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use embassy_time::{Duration, Timer};
use esp_hal::dma_rx_buffer;
use esp_hal::gpio::AnyPin;
use esp_hal::i2s::master::{I2s, PdmConfig, PdmRxConfig, PdmSlotMode};
use esp_hal::i2s::pdm::PdmSlotMask;
use esp_hal::peripherals::{DMA_CH0, I2S0};
use esp_hal::time::Rate;
use papermono_log::{
    MicSample, MIC_REPORT_MS, PCM_DUMP_NO_TONE_HZ, PCM_WINDOW_SAMPLES, TONE_DUMP_WINDOWS,
};

use crate::cdc;

/// UserDemo `hal_mic.cpp` intent: 16 kHz.
const SAMPLE_HZ: u32 = 16_000;
const WINDOW_BYTES: usize = PCM_WINDOW_SAMPLES * 2;

static LAST_RMS: AtomicU32 = AtomicU32::new(0);
static LAST_PEAK: AtomicU32 = AtomicU32::new(0);
static HAVE_SAMPLE: AtomicBool = AtomicBool::new(false);
static TONE_CAPTURE: AtomicU32 = AtomicU32::new(0);

/// Last energy window. Reprinted on the 10 s `hello` period.
pub fn last() -> Option<MicSample> {
    if !HAVE_SAMPLE.load(Ordering::Relaxed) {
        return None;
    }
    Some(MicSample {
        rms: LAST_RMS.load(Ordering::Relaxed),
        peak: LAST_PEAK.load(Ordering::Relaxed),
    })
}

/// Dump [`TONE_DUMP_WINDOWS`] after the next PDM windows (BUTTON A).
pub fn ask_tone() {
    TONE_CAPTURE.store(TONE_DUMP_WINDOWS, Ordering::Relaxed);
}

fn store(sample: MicSample) {
    LAST_RMS.store(sample.rms, Ordering::Relaxed);
    LAST_PEAK.store(sample.peak, Ordering::Relaxed);
    HAVE_SAMPLE.store(true, Ordering::Relaxed);
    cdc::mic(&sample);
}

#[embassy_executor::task]
pub async fn run(
    i2s0: I2S0<'static>,
    dma: DMA_CH0<'static>,
    clk: AnyPin<'static>,
    din: AnyPin<'static>,
) {
    let mut rx_cfg = PdmRxConfig::new_pcm_default(Rate::from_hz(SAMPLE_HZ), PdmSlotMode::Mono);
    // UserDemo `input_only_right`. HAL mono default is LEFT.
    rx_cfg.slot.slot_mask = PdmSlotMask::RIGHT;
    let pdm_cfg = PdmConfig::rx_only(rx_cfg);

    let Ok(i2s) = I2s::new_pdm(i2s0, dma, pdm_cfg) else {
        return;
    };
    let mut rx = i2s.into_async().i2s_rx.with_clk(clk).with_din(din).build();

    let Ok(mut buffer) = dma_rx_buffer!(WINDOW_BYTES) else {
        return;
    };

    loop {
        buffer.set_length(WINDOW_BYTES);
        let dump = TONE_CAPTURE.load(Ordering::Relaxed) > 0;
        match rx.read(buffer) {
            Ok(transfer) => {
                let (status, next, filled) = transfer.wait_async().await;
                rx = next;
                buffer = filled;
                if status.is_ok() {
                    emit_window(buffer.as_slice(), dump);
                    if dump {
                        let _ = TONE_CAPTURE.fetch_sub(1, Ordering::Relaxed);
                    }
                }
            }
            Err((_, next, returned)) => {
                rx = next;
                buffer = returned;
            }
        }
        if TONE_CAPTURE.load(Ordering::Relaxed) == 0 {
            Timer::after(Duration::from_millis(u64::from(MIC_REPORT_MS))).await;
        }
    }
}

fn emit_window(bytes: &[u8], dump_pcm: bool) {
    let n = bytes.len() / 2;
    if n == 0 {
        return;
    }
    let mut samples = [0i16; PCM_WINDOW_SAMPLES];
    for (i, chunk) in bytes.chunks_exact(2).take(n).enumerate() {
        samples[i] = i16::from_le_bytes([chunk[0], chunk[1]]);
    }
    let samples = &samples[..n.min(PCM_WINDOW_SAMPLES)];
    let (rms, peak) = pcm_energy(samples);
    store(MicSample { rms, peak });
    if dump_pcm {
        cdc::mic_pcm(PCM_DUMP_NO_TONE_HZ, samples);
    }
}

fn pcm_energy(samples: &[i16]) -> (u32, u32) {
    if samples.is_empty() {
        return (0, 0);
    }
    let mut sum_sq: u64 = 0;
    let mut peak: u32 = 0;
    for &sample in samples {
        let abs = u32::from(sample.unsigned_abs());
        if abs > peak {
            peak = abs;
        }
        sum_sq = sum_sq.saturating_add(u64::from(abs) * u64::from(abs));
    }
    let rms = isqrt(sum_sq / samples.len() as u64) as u32;
    (rms, peak)
}

fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = x.saturating_add(n / x) / 2;
    }
    x
}
