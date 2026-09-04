//! Host-only `vet-idle-log`. Parse lives in `papermono-log`.
//! No USB lock, no port. There is no `learn-uart` CLI.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use papermono_log::parse::{records, LineKind, Record};
use papermono_log::{IMAGE, IMAGE_EMBASSY};
use serde::Serialize;

use crate::Error;

/// Default SKU token in `hello sku=`.
pub const DEFAULT_SKU: &str = "C153-Lite";

/// Flags for [`vet_idle_log`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VetIdleLogArgs<'a> {
    /// Expected `hello image=` (`simple-debug` or `embassy-debug`).
    pub image: &'a str,
    /// Expected `hello sku=`.
    pub sku: &'a str,
    /// Allow `edge` and non-idle buttons (grammar check a busy capture).
    pub allow_activity: bool,
}

impl Default for VetIdleLogArgs<'static> {
    fn default() -> Self {
        Self {
            image: IMAGE,
            sku: DEFAULT_SKU,
            allow_activity: false,
        }
    }
}

/// Discover kinds, fields, and `hello` / `hb` periods from a capture.
pub fn learn_uart(path: &Path, json: bool) -> Result<(), Error> {
    let text = fs::read_to_string(path)?;
    let report = LearnReport::from_text(&text);
    if json {
        serde_json::to_writer_pretty(std::io::stdout(), &report)?;
        println!();
    } else {
        report.print();
    }
    Ok(())
}

/// Success when the log is this image at idle (or `--allow-activity`).
pub fn vet_idle_log(path: &Path, args: VetIdleLogArgs<'_>) -> Result<(), Error> {
    let text = fs::read_to_string(path)?;
    vet_idle_text(&text, args)
}

fn vet_idle_text(text: &str, args: VetIdleLogArgs<'_>) -> Result<(), Error> {
    validate_image(args.image)?;
    let recs: Vec<Record<'_>> = records(text).collect();
    if recs.is_empty() {
        return Err(Error::Device(
            "vet-idle-log: no simple-debug: records".into(),
        ));
    }

    if recs.iter().any(Record::mentions_mac) {
        return Err(Error::Device(
            "vet-idle-log: refuse mac / iserial fields".into(),
        ));
    }

    let mut hello_ok = false;
    let mut saw_hb = false;
    let mut saw_edge = false;
    let mut idle_buttons = true;

    for rec in &recs {
        match rec.kind {
            LineKind::Hello => {
                if rec.field("image") == Some(args.image) && rec.field("sku") == Some(args.sku) {
                    hello_ok = true;
                }
            }
            LineKind::Hb => {
                saw_hb = true;
                if rec.field("btn_a") != Some("1") || rec.field("btn_b") != Some("1") {
                    idle_buttons = false;
                }
            }
            LineKind::Edge => saw_edge = true,
            _ => {}
        }
    }

    if !hello_ok {
        return Err(Error::Device(format!(
            "vet-idle-log: need hello image={} sku={}",
            args.image, args.sku
        )));
    }
    if !saw_hb {
        return Err(Error::Device("vet-idle-log: need at least one hb".into()));
    }
    if !args.allow_activity && saw_edge {
        return Err(Error::Device(
            "vet-idle-log: edge is activity; pass --allow-activity".into(),
        ));
    }
    if !args.allow_activity && !idle_buttons {
        return Err(Error::Device(
            "vet-idle-log: hb buttons are not idle (want btn_a=1 btn_b=1)".into(),
        ));
    }

    println!("vet-idle-log: ok");
    Ok(())
}

fn validate_image(image: &str) -> Result<(), Error> {
    if image == IMAGE || image == IMAGE_EMBASSY {
        Ok(())
    } else {
        Err(Error::Device(format!(
            "vet-idle-log: unknown image {image:?} (want {IMAGE} or {IMAGE_EMBASSY})"
        )))
    }
}

#[derive(Debug, Serialize)]
struct LearnReport {
    records: usize,
    kinds: Vec<String>,
    fields: BTreeMap<String, Vec<String>>,
    fingerprint: Fingerprint,
    hb_period_s: Option<u32>,
    hello_period_s: Option<u32>,
    mentions_mac: bool,
}

#[derive(Debug, Serialize)]
struct Fingerprint {
    image: Option<String>,
    sku: Option<String>,
}

impl LearnReport {
    fn from_text(text: &str) -> Self {
        let recs: Vec<Record<'_>> = records(text).collect();
        let mut kinds = BTreeSet::new();
        let mut fields: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut hb_t = Vec::new();
        let mut hello_t = Vec::new();
        let mut image = None;
        let mut sku = None;
        let mut mentions_mac = false;

        for rec in &recs {
            let kind = kind_name(rec.kind);
            kinds.insert(kind.to_string());
            let entry = fields.entry(kind.to_string()).or_default();
            for token in rec.body.split_ascii_whitespace() {
                if let Some((name, _)) = token.split_once('=') {
                    entry.insert(name.to_string());
                }
            }
            mentions_mac |= rec.mentions_mac();
            match rec.kind {
                LineKind::Hb => {
                    if let Some(t) = rec.field("t").and_then(|s| s.parse().ok()) {
                        hb_t.push(t);
                    }
                }
                LineKind::Hello => {
                    if image.is_none() {
                        image = rec.field("image").map(str::to_string);
                    }
                    if sku.is_none() {
                        sku = rec.field("sku").map(str::to_string);
                    }
                    if let Some(t) = rec.field("t").and_then(|s| s.parse().ok()) {
                        hello_t.push(t);
                    }
                }
                _ => {}
            }
        }

        Self {
            records: recs.len(),
            kinds: kinds.into_iter().collect(),
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
            fingerprint: Fingerprint { image, sku },
            hb_period_s: typical_period(&hb_t),
            hello_period_s: typical_period(&hello_t),
            mentions_mac,
        }
    }

    fn print(&self) {
        println!("classify: records={}", self.records);
        println!("kinds: {}", self.kinds.join(" "));
        for (kind, names) in &self.fields {
            println!("{kind} fields: {}", names.join(" "));
        }
        match (&self.fingerprint.image, &self.fingerprint.sku) {
            (Some(image), Some(sku)) => {
                println!("fingerprint: image={image} sku={sku}");
            }
            (Some(image), None) => println!("fingerprint: image={image}"),
            (None, Some(sku)) => println!("fingerprint: sku={sku}"),
            (None, None) => println!("fingerprint: (none)"),
        }
        print_period("hb", self.hb_period_s);
        print_period("hello", self.hello_period_s);
        if self.mentions_mac {
            println!("mentions_mac: yes (refused identity fields present)");
        }
    }
}

fn print_period(kind: &str, period: Option<u32>) {
    match period {
        Some(s) => println!("{kind} period_s: {s} (from t=)"),
        None => println!("{kind} period_s: unknown"),
    }
}

fn kind_name(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Hello => "hello",
        LineKind::Git => "git",
        LineKind::Gpio => "gpio",
        LineKind::Hb => "hb",
        LineKind::Edge => "edge",
        LineKind::I2c => "i2c",
        LineKind::Touch => "touch",
        LineKind::Mic => "mic",
        LineKind::Pcm => "pcm",
        LineKind::Panel => "panel",
        LineKind::Scene => "scene",
        LineKind::Lamp => "lamp",
        LineKind::Leftover => "leftover",
        LineKind::Wifi => "wifi",
        LineKind::Ble => "ble",
        LineKind::Charge => "charge",
        LineKind::Sleep => "sleep",
        LineKind::Wake => "wake",
        LineKind::Snowflake => "snowflake",
        LineKind::Pair => "pair",
        LineKind::WifiSurvey => "wifi_survey",
        LineKind::WifiAp => "wifi_ap",
        LineKind::WifiHttp => "wifi_http",
        LineKind::Imu => "imu",
        LineKind::Unknown => "unknown",
    }
}

fn typical_period(samples: &[u32]) -> Option<u32> {
    if samples.len() < 2 {
        return None;
    }
    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for window in samples.windows(2) {
        if window[1] > window[0] {
            *counts.entry(window[1] - window[0]).or_insert(0) += 1;
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(d, _)| d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: &str = "\
simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: git=abc dirty=0
simple-debug: gpio boot=1 pmic_irq=0 tp=0 ioe=1 busy=0
simple-debug: hb t=0 btn_a=1 btn_b=1
simple-debug: hello t=10 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: hb t=1 btn_a=1 btn_b=1
simple-debug: hb t=2 btn_a=1 btn_b=1
";

    const GLUED: &str =
        "simple-debug: hb t=1 btn_a=1 btn_b=1simple-debug: hb t=27 btn_a=1 btn_b=1\n";

    const BUSY: &str = "\
simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: hb t=0 btn_a=1 btn_b=1
simple-debug: edge t_ms=1250 btn_a=1->0
simple-debug: hb t=1 btn_a=0 btn_b=1
";

    #[test]
    fn idle_log_vets() {
        vet_idle_text(IDLE, VetIdleLogArgs::default()).unwrap();
    }

    #[test]
    fn glued_heartbeats_are_idle() {
        let text = format!(
            "simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on\n{GLUED}"
        );
        vet_idle_text(&text, VetIdleLogArgs::default()).unwrap();
    }

    #[test]
    fn button_session_fails_idle() {
        let err = vet_idle_text(BUSY, VetIdleLogArgs::default()).unwrap_err();
        assert!(err.to_string().contains("edge"));
    }

    #[test]
    fn allow_activity_accepts_edges() {
        vet_idle_text(
            BUSY,
            VetIdleLogArgs {
                allow_activity: true,
                ..VetIdleLogArgs::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn embassy_image_needs_flag() {
        let text = IDLE.replace("image=simple-debug", "image=embassy-debug");
        assert!(vet_idle_text(&text, VetIdleLogArgs::default()).is_err());
        vet_idle_text(
            &text,
            VetIdleLogArgs {
                image: IMAGE_EMBASSY,
                ..VetIdleLogArgs::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn mac_is_refused() {
        let text = IDLE.replace("hb t=0 btn_a=1 btn_b=1", "hb t=0 btn_a=1 btn_b=1 mac=aa");
        let err = vet_idle_text(&text, VetIdleLogArgs::default()).unwrap_err();
        assert!(err.to_string().contains("mac"));
    }

    #[test]
    fn learn_finds_glued_kinds_and_periods() {
        let report = LearnReport::from_text(IDLE);
        assert!(report.kinds.iter().any(|k| k == "hello"));
        assert!(report.kinds.iter().any(|k| k == "hb"));
        assert_eq!(report.fingerprint.image.as_deref(), Some("simple-debug"));
        assert_eq!(report.fingerprint.sku.as_deref(), Some("C153-Lite"));
        assert_eq!(report.hello_period_s, Some(10));
        assert_eq!(report.hb_period_s, Some(1));
        assert!(!report.mentions_mac);

        let glued = LearnReport::from_text(GLUED);
        assert_eq!(glued.records, 2);
        assert_eq!(glued.kinds, ["hb"]);
    }
}
