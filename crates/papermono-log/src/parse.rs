//! Split CDC captures on `simple-debug:`. No heap.
//!
//! USB-Serial/JTAG may glue two records by dropping `\n`. Search for the
//! prefix; do not require a newline between them.

/// On-wire mark, including the colon. [`crate::LOG_PREFIX`] is the token
/// before that colon.
const RECORD_MARK: &str = "simple-debug:";

/// Kind of one `simple-debug:` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// Repeating identity (`hello`).
    Hello,
    /// Git stamp (`git=`).
    Git,
    /// Input sample (`gpio`).
    Gpio,
    /// 1 Hz buttons (`hb`).
    Hb,
    /// Button transition (`edge`).
    Edge,
    /// I2C ACK/NAK sample (`i2c`).
    I2c,
    /// Touch INT / contacts (`touch`).
    Touch,
    /// PDM energy (`mic`).
    Mic,
    /// PCM sample row (`pcm`).
    Pcm,
    /// Panel refresh stamp (`panel`).
    Panel,
    /// Card token (`scene=`).
    Scene,
    /// Frontlight PWM duty (`lamp=`).
    Lamp,
    /// Lite leftover MCU inputs (`leftover`).
    Leftover,
    /// Wi-Fi scan count (`wifi n=`).
    Wifi,
    /// BLE scan count (`ble n=`).
    Ble,
    /// Gated charge sample (`charge vbat=`).
    Charge,
    /// Sleep arm or abort (`sleep rtc=` / `sleep abort`).
    Sleep,
    /// Wake source (`wake src=`).
    Wake,
    /// Prefix matched, first token unknown.
    Unknown,
}

/// One record after the prefix, whitespace trimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record<'a> {
    /// Classified kind.
    pub kind: LineKind,
    /// Text after `simple-debug:` with leading/trailing ASCII space stripped.
    pub body: &'a str,
}

/// Walk `text` yielding one record per prefix, including glued lines.
pub fn records(text: &str) -> RecordIter<'_> {
    RecordIter { rest: text }
}

/// Iterator over [`Record`] values in a capture.
#[derive(Debug, Clone)]
pub struct RecordIter<'a> {
    rest: &'a str,
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Record<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.rest.find(RECORD_MARK)?;
        let after_prefix = start + RECORD_MARK.len();
        let tail = &self.rest[after_prefix..];
        let rel = tail.find(RECORD_MARK).unwrap_or(tail.len());
        let raw = &tail[..rel];
        self.rest = &tail[rel..];
        let body = trim_ascii(raw);
        Some(Record {
            kind: classify(body),
            body,
        })
    }
}

impl Record<'_> {
    /// Value of `key=` in this record, if present.
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&str> {
        field(self.body, key)
    }

    /// True when the record text contains `mac` as a whole-token prefix.
    #[must_use]
    pub fn mentions_mac(&self) -> bool {
        mentions_mac(self.body)
    }
}

/// First `key=value` token in `body` whose key matches.
#[must_use]
pub fn field<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    for token in body.split_ascii_whitespace() {
        let Some((name, value)) = token.split_once('=') else {
            continue;
        };
        if name == key {
            return Some(value);
        }
    }
    None
}

/// Classify the first token of a record body.
#[must_use]
pub fn classify(body: &str) -> LineKind {
    let token = body.split_ascii_whitespace().next().unwrap_or("");
    if token.starts_with("git=") {
        return LineKind::Git;
    }
    if token.starts_with("scene=") {
        return LineKind::Scene;
    }
    if token.starts_with("lamp=") {
        return LineKind::Lamp;
    }
    match token {
        "hello" => LineKind::Hello,
        "gpio" => LineKind::Gpio,
        "hb" => LineKind::Hb,
        "edge" => LineKind::Edge,
        "i2c" => LineKind::I2c,
        "touch" => LineKind::Touch,
        "mic" => LineKind::Mic,
        "pcm" => LineKind::Pcm,
        "panel" => LineKind::Panel,
        "leftover" => LineKind::Leftover,
        "wifi" => LineKind::Wifi,
        "ble" => LineKind::Ble,
        "charge" => LineKind::Charge,
        "sleep" => LineKind::Sleep,
        "wake" => LineKind::Wake,
        _ => LineKind::Unknown,
    }
}

fn trim_ascii(s: &str) -> &str {
    s.trim_matches(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r')
}

fn mentions_mac(body: &str) -> bool {
    for token in body.split_ascii_whitespace() {
        let name = token.split_once('=').map(|(n, _)| n).unwrap_or(token);
        if eq_ignore_ascii(name, "mac")
            || eq_ignore_ascii(name, "iserial")
            || eq_ignore_ascii(name, "bssid")
            || eq_ignore_ascii(name, "irk")
        {
            return true;
        }
    }
    false
}

fn eq_ignore_ascii(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .all(|(l, r)| l.eq_ignore_ascii_case(&r))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn glued_heartbeats_split_on_prefix() {
        let text = "simple-debug: hb t=1 btn_a=1 btn_b=1simple-debug: hb t=27 btn_a=1 btn_b=1\n";
        let recs: std::vec::Vec<_> = records(text).collect();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].kind, LineKind::Hb);
        assert_eq!(recs[0].field("t"), Some("1"));
        assert_eq!(recs[1].field("t"), Some("27"));
    }

    #[test]
    fn hello_fields_and_no_mac() {
        let text = "simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on\r\n";
        let rec = records(text).next().unwrap();
        assert_eq!(rec.kind, LineKind::Hello);
        assert_eq!(rec.field("image"), Some("simple-debug"));
        assert_eq!(rec.field("sku"), Some("C153-Lite"));
        assert!(!rec.mentions_mac());
    }

    #[test]
    fn git_kind_from_git_equals() {
        let rec = records("simple-debug: git=abc dirty=0\n").next().unwrap();
        assert_eq!(rec.kind, LineKind::Git);
        assert_eq!(rec.field("git"), Some("abc"));
    }

    #[test]
    fn mac_field_is_flagged() {
        let rec = records("simple-debug: hb t=1 mac=aa\n").next().unwrap();
        assert!(rec.mentions_mac());
    }

    #[test]
    fn later_kinds_classify() {
        assert_eq!(
            records("simple-debug: i2c pm1=1 nfc=0\n")
                .next()
                .unwrap()
                .kind,
            LineKind::I2c
        );
        assert_eq!(
            records("simple-debug: touch int=0 n=0\n")
                .next()
                .unwrap()
                .kind,
            LineKind::Touch
        );
        assert_eq!(
            records("simple-debug: mic rms=1 peak=2\n")
                .next()
                .unwrap()
                .kind,
            LineKind::Mic
        );
        assert_eq!(
            records("simple-debug: pcm 000 120 -30\n")
                .next()
                .unwrap()
                .kind,
            LineKind::Pcm
        );
        assert_eq!(
            records("simple-debug: panel mode=otp_gray w=800 h=480 busy_rose=1\n")
                .next()
                .unwrap()
                .kind,
            LineKind::Panel
        );
        assert_eq!(
            records("simple-debug: scene=splash\n").next().unwrap().kind,
            LineKind::Scene
        );
        assert_eq!(
            records("simple-debug: lamp=1024\n").next().unwrap().kind,
            LineKind::Lamp
        );
        assert_eq!(
            records("simple-debug: leftover lora_irq=0 nfc_irq=1 sx_busy=0\n")
                .next()
                .unwrap()
                .kind,
            LineKind::Leftover
        );
        assert_eq!(
            records("simple-debug: wifi n=12\n").next().unwrap().kind,
            LineKind::Wifi
        );
        assert_eq!(
            records("simple-debug: ble n=4\n").next().unwrap().kind,
            LineKind::Ble
        );
        assert_eq!(
            records("simple-debug: charge vbat=3921 vin=5080 src=05 chg_en=1 ip=1 then=0\n")
                .next()
                .unwrap()
                .kind,
            LineKind::Charge
        );
        assert_eq!(
            records("simple-debug: sleep rtc=10\n").next().unwrap().kind,
            LineKind::Sleep
        );
        assert_eq!(
            records("simple-debug: wake src=20\n").next().unwrap().kind,
            LineKind::Wake
        );
    }

    #[test]
    fn bssid_and_irk_are_flagged() {
        assert!(records("simple-debug: wifi bssid=aa\n")
            .next()
            .unwrap()
            .mentions_mac());
        assert!(records("simple-debug: ble irk=bb\n")
            .next()
            .unwrap()
            .mentions_mac());
    }
}
