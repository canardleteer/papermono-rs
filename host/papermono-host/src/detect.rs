//! USB inventory (default) or flasher `--probe`.
//!
//! Default is sysfs / by-id inventory (no port open). `--probe` connects
//! with no DTR-as-download (prefer the power-button hold). Inventory
//! redacts the USB iSerial.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::device::DeviceIo;
use crate::identity::{
    parse_board_info, parse_usb_serial_from_port, qinheng_marker, sha256_text, BoardInfo,
};
use crate::Error;

/// Espressif USB VID (native USB-Serial/JTAG).
pub const ESPRESSIF_VID: u16 = 0x303A;
/// USB JTAG/serial debug unit product id (Lite run mode).
pub const ESPRESSIF_JTAG_PID: u16 = 0x1001;
/// QinHeng CH343P (`lsusb` `ID 1a86:55d3`) — wrong product.
pub const QINHENG_VID: u16 = 0x1A86;
/// QinHeng “USB Single Serial” product id.
pub const QINHENG_PID: u16 = 0x55D3;

/// How a USB serial node relates to PaperMono / Lite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    /// Espressif `303a:1001` USB-Serial/JTAG.
    PaperMonoUsb,
    /// QinHeng CH343 (Sticky / other boards).
    QinHengCh343,
    /// Some other USB-serial adapter.
    Other,
}

/// One discovered serial node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Stable by-id path when udev created one.
    pub by_id: Option<PathBuf>,
    /// Kernel ACM/USB-serial name (`ttyACM1`, …).
    pub tty_name: Option<String>,
    /// USB iSerial from by-id or sysfs (never print this).
    pub usb_serial: Option<String>,
    /// USB vendor id.
    pub vid: Option<u16>,
    /// USB product id.
    pub pid: Option<u16>,
    /// USB product string.
    pub product: Option<String>,
    /// Classification.
    pub kind: PortKind,
}

impl Candidate {
    /// Preferred `ESPFLASH_PORT` value (by-id if present).
    #[must_use]
    pub fn preferred_port(&self) -> Option<String> {
        if let Some(path) = &self.by_id {
            return Some(path.display().to_string());
        }
        self.tty_name
            .as_ref()
            .map(|tty| host_dev_dir().join(tty).display().to_string())
    }
}

/// Host device directory (`/dev`) without embedding a gated tty path.
#[must_use]
pub fn host_dev_dir() -> PathBuf {
    PathBuf::from("/dev")
}

fn serial_by_id_dir() -> PathBuf {
    host_dev_dir().join("serial").join("by-id")
}

fn sys_class_tty_dir() -> PathBuf {
    PathBuf::from("/sys/class/tty")
}

/// Classify from USB ids and/or a by-id path string.
#[must_use]
pub fn classify(vid: Option<u16>, pid: Option<u16>, port_path: Option<&str>) -> PortKind {
    if vid == Some(QINHENG_VID) && pid == Some(QINHENG_PID) {
        return PortKind::QinHengCh343;
    }
    if port_path.is_some_and(|p| p.contains(qinheng_marker())) {
        return PortKind::QinHengCh343;
    }
    if vid == Some(ESPRESSIF_VID) && pid == Some(ESPRESSIF_JTAG_PID) {
        return PortKind::PaperMonoUsb;
    }
    if port_path.and_then(parse_usb_serial_from_port).is_some() {
        return PortKind::PaperMonoUsb;
    }
    if vid == Some(ESPRESSIF_VID) {
        return PortKind::PaperMonoUsb;
    }
    PortKind::Other
}

fn parse_hex_u16(raw: &str) -> Option<u16> {
    u16::from_str_radix(raw.trim().trim_start_matches("0x"), 16).ok()
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// USB device that backs an ACM node (`/dev/bus/usb/{busnum}/{devnum}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbDeviceKey {
    /// `busnum` from sysfs.
    pub busnum: u8,
    /// `devnum` from sysfs (USB device address).
    pub devnum: u8,
    /// USB vendor id.
    pub vid: u16,
    /// USB product id.
    pub pid: u16,
}

/// Map `--port` / by-id / ACM name to the USB device, without opening the TTY.
pub fn usb_device_key_for_port(port: &str) -> Result<UsbDeviceKey, Error> {
    usb_device_key_from(port, &sys_class_tty_dir())
}

/// Testable [`usb_device_key_for_port`].
pub fn usb_device_key_from(port: &str, sys_tty: &Path) -> Result<UsbDeviceKey, Error> {
    let tty = tty_name_for_port(port)
        .ok_or_else(|| Error::Device(format!("cannot map {} to a ttyACM/ttyUSB name", port)))?;
    usb_device_key_for_tty(sys_tty, &tty).ok_or_else(|| {
        Error::Device(format!(
            "no USB busnum/devnum in sysfs for {tty}; is the unit still plugged in?"
        ))
    })
}

fn tty_name_for_port(port: &str) -> Option<String> {
    if let Some(name) = port_file_name(port) {
        if name.starts_with("ttyACM") || name.starts_with("ttyUSB") {
            return Some(name.to_string());
        }
    }
    tty_from_by_id_link(Path::new(port))
}

fn usb_device_key_for_tty(sys_tty: &Path, tty: &str) -> Option<UsbDeviceKey> {
    let info = usb_sysfs_for_tty(sys_tty, tty)?;
    Some(UsbDeviceKey {
        busnum: info.busnum?,
        devnum: info.devnum?,
        vid: info.vid?,
        pid: info.pid?,
    })
}

struct UsbSysfs {
    vid: Option<u16>,
    pid: Option<u16>,
    product: Option<String>,
    serial: Option<String>,
    busnum: Option<u8>,
    devnum: Option<u8>,
}

fn parse_u8_dec(raw: &str) -> Option<u8> {
    raw.trim().parse().ok()
}

fn usb_sysfs_for_tty(sys_tty: &Path, tty: &str) -> Option<UsbSysfs> {
    let start = sys_tty.join(tty).join("device");
    let mut cur = fs::canonicalize(&start).ok()?;
    for _ in 0..10 {
        let vendor = cur.join("idVendor");
        if vendor.is_file() {
            return Some(UsbSysfs {
                vid: read_trimmed(&vendor).and_then(|s| parse_hex_u16(&s)),
                pid: read_trimmed(&cur.join("idProduct")).and_then(|s| parse_hex_u16(&s)),
                product: read_trimmed(&cur.join("product")),
                serial: read_trimmed(&cur.join("serial")),
                busnum: read_trimmed(&cur.join("busnum")).and_then(|s| parse_u8_dec(&s)),
                devnum: read_trimmed(&cur.join("devnum")).and_then(|s| parse_u8_dec(&s)),
            });
        }
        cur = cur.parent()?.to_path_buf();
    }
    None
}

fn usb_info_for_tty(
    sys_tty: &Path,
    tty: &str,
) -> (Option<u16>, Option<u16>, Option<String>, Option<String>) {
    match usb_sysfs_for_tty(sys_tty, tty) {
        Some(info) => (info.vid, info.pid, info.product, info.serial),
        None => (None, None, None, None),
    }
}

fn tty_from_by_id_link(link: &Path) -> Option<String> {
    let target = fs::read_link(link).ok()?;
    target
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_string)
        .filter(|n| n.starts_with("ttyACM") || n.starts_with("ttyUSB"))
}

/// Scan udev by-id plus ACM/USB-serial nodes (paths constructed, not literals).
pub fn scan() -> Result<Vec<Candidate>, Error> {
    scan_from(&serial_by_id_dir(), &host_dev_dir(), &sys_class_tty_dir())
}

/// Testable scan against fixture directories (no live `/dev` required).
pub fn scan_from(
    by_id_dir: &Path,
    dev_dir: &Path,
    sys_tty: &Path,
) -> Result<Vec<Candidate>, Error> {
    let mut by_tty: BTreeMap<String, Candidate> = BTreeMap::new();
    let mut by_id_only = Vec::new();

    if by_id_dir.is_dir() {
        for entry in fs::read_dir(by_id_dir)? {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_string_lossy();
            let tty = tty_from_by_id_link(&path);
            let usb_from_name = parse_usb_serial_from_port(&path_str);
            let (vid, pid, product, sys_serial) = tty
                .as_deref()
                .map(|tty| usb_info_for_tty(sys_tty, tty))
                .unwrap_or((None, None, None, None));
            let usb_serial = usb_from_name.or(sys_serial);
            let kind = classify(vid, pid, Some(path_str.as_ref()));
            let cand = Candidate {
                by_id: Some(path),
                tty_name: tty.clone(),
                usb_serial,
                vid,
                pid,
                product,
                kind,
            };
            if let Some(tty) = tty {
                by_tty.insert(tty, cand);
            } else {
                by_id_only.push(cand);
            }
        }
    }

    if dev_dir.is_dir() {
        for entry in fs::read_dir(dev_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if !name.starts_with("ttyACM") && !name.starts_with("ttyUSB") {
                continue;
            }
            if by_tty.contains_key(name) {
                continue;
            }
            let (vid, pid, product, serial) = usb_info_for_tty(sys_tty, name);
            let kind = classify(vid, pid, None);
            by_tty.insert(
                name.to_string(),
                Candidate {
                    by_id: None,
                    tty_name: Some(name.to_string()),
                    usb_serial: serial,
                    vid,
                    pid,
                    product,
                    kind,
                },
            );
        }
    }

    let mut out: Vec<_> = by_tty.into_values().collect();
    out.extend(by_id_only);
    out.sort_by_key(|a| a.preferred_port());
    Ok(out)
}

/// Strip a USB iSerial or MAC-shaped token from a path before printing.
#[must_use]
pub fn redact_identity_in_path(path: &str, usb_serial: Option<&str>) -> String {
    let mut out = path.to_string();
    if let Some(serial) = usb_serial {
        if !serial.is_empty() {
            out = out.replace(serial, "<redacted>");
        }
    }
    out
}

fn kind_label(kind: PortKind) -> &'static str {
    match kind {
        PortKind::PaperMonoUsb => "PaperMono USB-Serial/JTAG (Espressif 303a:1001)",
        PortKind::QinHengCh343 => "QinHeng CH343 (1a86:55d3) — not this product",
        PortKind::Other => "other USB-serial",
    }
}

fn listed_for_inventory(candidates: &[Candidate], all_devices: bool) -> Vec<&Candidate> {
    if all_devices {
        candidates.iter().collect()
    } else {
        candidates
            .iter()
            .filter(|c| c.kind == PortKind::PaperMonoUsb)
            .collect()
    }
}

/// Print inventory. Does not open a serial port. Redacts iSerial.
pub fn print_inventory(candidates: &[Candidate], all_devices: bool) {
    let listed = listed_for_inventory(candidates, all_devices);
    let hidden = candidates.len().saturating_sub(listed.len());
    let papermono: Vec<_> = candidates
        .iter()
        .filter(|c| c.kind == PortKind::PaperMonoUsb)
        .collect();

    if listed.is_empty() {
        if candidates.is_empty() {
            println!("detect-connected: no USB-serial nodes found");
        } else {
            println!("detect-connected: no Espressif 303a:1001 PaperMono USB classified");
            if hidden > 0 {
                println!("({hidden} other USB-serial node(s) omitted; pass --all-devices)");
            }
        }
        return;
    }

    if all_devices {
        println!("detect-connected: {} USB-serial node(s)", listed.len());
    } else {
        println!(
            "detect-connected: {} PaperMono USB-Serial/JTAG node(s)",
            listed.len()
        );
    }
    for (i, c) in listed.iter().enumerate() {
        println!("{}. {}", i + 1, kind_label(c.kind));
        if let Some(p) = &c.by_id {
            println!(
                "   by-id: {}",
                redact_identity_in_path(&p.display().to_string(), c.usb_serial.as_deref())
            );
        } else {
            println!("   by-id: (none; unstable ACM node)");
        }
        match &c.tty_name {
            Some(t) => println!("   kernel: {t}"),
            None => println!("   kernel: (unknown)"),
        }
        match (c.vid, c.pid) {
            (Some(v), Some(p)) => println!("   vid:pid: {v:04x}:{p:04x}"),
            _ => println!("   vid:pid: (not in sysfs)"),
        }
        if let Some(product) = &c.product {
            println!("   product: {product}");
        }
        if c.usb_serial.is_some() {
            println!("   usb serial: present");
        } else {
            println!("   usb serial: (none)");
        }
        if let Some(port) = c.preferred_port() {
            let shown = redact_identity_in_path(&port, c.usb_serial.as_deref());
            if c.kind == PortKind::PaperMonoUsb {
                println!("   ESPFLASH_PORT={shown}");
            } else {
                println!("   path: {shown}");
            }
        }
    }
    match papermono.len() {
        1 => {
            if let Some(port) = papermono[0].preferred_port() {
                let shown = redact_identity_in_path(&port, papermono[0].usb_serial.as_deref());
                println!("suggested: export ESPFLASH_PORT={shown}");
                if shown.contains("<redacted>") {
                    println!(
                        "suggested: copy the real by-id node from /dev/serial/by-id/ locally \
                         (iSerial omitted here)"
                    );
                }
            }
        }
        n if n > 1 => println!("multiple PaperMono USB nodes; pass --port to --probe"),
        _ => println!("no Espressif 303a:1001 PaperMono USB classified"),
    }
    if hidden > 0 {
        println!("({hidden} other USB-serial node(s) omitted; pass --all-devices)");
    }
}

fn port_file_name(port: &str) -> Option<&str> {
    Path::new(port).file_name()?.to_str()
}

fn candidate_matches(candidate: &Candidate, port: &str) -> bool {
    if let Some(by_id) = &candidate.by_id {
        if by_id == Path::new(port) {
            return true;
        }
        if by_id.file_name().and_then(|n| n.to_str()) == port_file_name(port) {
            return true;
        }
    }
    if candidate.preferred_port().as_deref() == Some(port) {
        return true;
    }
    if let Some(tty) = &candidate.tty_name {
        if port_file_name(port) == Some(tty.as_str()) {
            return true;
        }
    }
    matches!(
        (
            parse_usb_serial_from_port(port),
            candidate.usb_serial.as_deref(),
        ),
        (Some(from_port), Some(from_usb)) if from_port == from_usb
    )
}

/// Refuse a port that is not Espressif `303a:1001`.
pub fn require_papermono_usb(port: &str) -> Result<(), Error> {
    require_papermono_usb_from(port, &scan()?, &sys_class_tty_dir())
}

/// Testable [require_papermono_usb] against a fixture inventory.
pub fn require_papermono_usb_from(
    port: &str,
    candidates: &[Candidate],
    sys_tty: &Path,
) -> Result<(), Error> {
    let (kind, vid, pid) =
        if let Some(found) = candidates.iter().find(|c| candidate_matches(c, port)) {
            (found.kind, found.vid, found.pid)
        } else {
            let tty = port_file_name(port)
                .filter(|name| name.starts_with("ttyACM") || name.starts_with("ttyUSB"));
            let (vid, pid, _, _) = tty
                .map(|tty| usb_info_for_tty(sys_tty, tty))
                .unwrap_or((None, None, None, None));
            (classify(vid, pid, Some(port)), vid, pid)
        };
    match kind {
        PortKind::PaperMonoUsb => Ok(()),
        PortKind::QinHengCh343 => Err(Error::QinHengCh343),
        PortKind::Other => {
            if vid.is_none() && pid.is_none() && parse_usb_serial_from_port(port).is_none() {
                Err(Error::UnclassifiedUsbPort)
            } else {
                Err(Error::NotPaperMonoUsb { vid, pid })
            }
        }
    }
}

fn pick_papermono_port(port: Option<String>, candidates: &[Candidate]) -> Result<String, Error> {
    if let Some(port) = port {
        return Ok(port);
    }
    let nodes: Vec<_> = candidates
        .iter()
        .filter(|c| c.kind == PortKind::PaperMonoUsb)
        .collect();
    match nodes.len() {
        0 => Err(Error::MissingPaperMonoUsb),
        1 => nodes[0].preferred_port().ok_or(Error::MissingPaperMonoUsb),
        _ => Err(Error::AmbiguousPaperMonoUsb),
    }
}

/// Pick `--port` / `ESPFLASH_PORT`, or the unique PaperMono USB node.
pub fn resolve_papermono_port(explicit: Option<String>) -> Result<String, Error> {
    resolve_papermono_port_from(explicit, &scan()?, &sys_class_tty_dir())
}

/// Testable [resolve_papermono_port].
pub fn resolve_papermono_port_from(
    explicit: Option<String>,
    candidates: &[Candidate],
    sys_tty: &Path,
) -> Result<String, Error> {
    let port = pick_papermono_port(explicit, candidates)?;
    require_papermono_usb_from(&port, candidates, sys_tty)?;
    Ok(port)
}

/// Flasher board-info plus USB serial hash from the port path / sysfs.
pub fn read_live_board<D: DeviceIo>(device: &D, port: &str) -> Result<(String, BoardInfo), Error> {
    let info_text = device.board_info(port)?;
    let mut board = parse_board_info(&info_text)?;
    if let Some(serial) = parse_usb_serial_from_port(port).or_else(|| usb_serial_from_sysfs(port)) {
        board.identity.usb_serial_sha256 = Some(sha256_text(&serial));
    }
    Ok((info_text, board))
}

fn usb_serial_from_sysfs(port: &str) -> Option<String> {
    let tty = tty_name_for_port(port)?;
    usb_sysfs_for_tty(&sys_class_tty_dir(), &tty)?.serial
}

/// Flasher board-info. Prefer download via power-button hold; no MAC printed.
pub fn probe<D: DeviceIo>(device: &D, port: &str) -> Result<(), Error> {
    let serial = parse_usb_serial_from_port(port);
    println!(
        "probe port: {}",
        redact_identity_in_path(port, serial.as_deref())
    );
    println!(
        "probe: prefer download via power-button hold (~2 s until red LED blinks). \
         DTR is not the primary path."
    );
    let (text, board) = read_live_board(device, port)?;
    print!("{text}");
    match board.flash_size_bytes {
        Some(bytes) => println!(
            "probe: measured flash {} bytes ({})",
            bytes, board.flash_size
        ),
        None => println!("probe: flash size unusable ({:?})", board.flash_size),
    }
    Ok(())
}

/// USB inventory, optionally `--probe`.
pub fn run<D: DeviceIo>(
    device: &D,
    probe_chip: bool,
    port: Option<String>,
    all_devices: bool,
) -> Result<(), Error> {
    let candidates = scan()?;
    print_inventory(&candidates, all_devices);
    let _ = io::stdout().flush();
    if probe_chip {
        let port = resolve_papermono_port_from(port, &candidates, &sys_class_tty_dir())?;
        let _uart = crate::uart_lock::try_acquire(&port, "detect-connected --probe")?;
        probe(device, &port)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::espressif_jtag_marker;
    use std::path::PathBuf;

    #[test]
    fn espressif_jtag_is_papermono() {
        assert_eq!(
            classify(Some(ESPRESSIF_VID), Some(ESPRESSIF_JTAG_PID), None),
            PortKind::PaperMonoUsb
        );
    }

    #[test]
    fn qinheng_is_refused() {
        assert_eq!(
            classify(Some(QINHENG_VID), Some(QINHENG_PID), None),
            PortKind::QinHengCh343
        );
    }

    #[test]
    fn by_id_name_classifies_without_sysfs() {
        let marker = espressif_jtag_marker();
        let path = format!("prefix/{marker}_TESTUSB-if00");
        assert_eq!(classify(None, None, Some(&path)), PortKind::PaperMonoUsb);
        assert_eq!(
            parse_usb_serial_from_port(&path).as_deref(),
            Some("TESTUSB")
        );
    }

    #[test]
    fn scan_by_id_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let by_id = tmp.path().join("by-id");
        let dev = tmp.path().join("dev");
        let sys_tty = tmp.path().join("sys-tty");
        fs::create_dir_all(&by_id).unwrap();
        fs::create_dir_all(&dev).unwrap();
        let marker = espressif_jtag_marker();
        let name = format!("{marker}_TESTUSB-if00");
        std::os::unix::fs::symlink("../../ttyACM9", by_id.join(&name)).unwrap();
        let found = scan_from(&by_id, &dev, &sys_tty).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, PortKind::PaperMonoUsb);
        assert_eq!(found[0].usb_serial.as_deref(), Some("TESTUSB"));
        assert_eq!(found[0].tty_name.as_deref(), Some("ttyACM9"));
    }

    fn write_sysfs_usb(sys_tty: &Path, tty: &str, vid: &str, pid: &str, serial: &str) {
        let usb = sys_tty.join(tty).join("usb");
        fs::create_dir_all(&usb).unwrap();
        fs::write(usb.join("idVendor"), vid).unwrap();
        fs::write(usb.join("idProduct"), pid).unwrap();
        fs::write(usb.join("product"), "USB JTAG/serial debug unit").unwrap();
        fs::write(usb.join("serial"), serial).unwrap();
        fs::write(usb.join("busnum"), "3").unwrap();
        fs::write(usb.join("devnum"), "14").unwrap();
        std::os::unix::fs::symlink("usb", sys_tty.join(tty).join("device")).unwrap();
    }

    #[test]
    fn scan_acm_node_from_sysfs_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let by_id = tmp.path().join("by-id");
        let dev = tmp.path().join("dev");
        let sys_tty = tmp.path().join("sys-tty");
        fs::create_dir_all(&by_id).unwrap();
        fs::create_dir_all(&dev).unwrap();
        fs::write(dev.join("ttyACM3"), b"").unwrap();
        write_sysfs_usb(&sys_tty, "ttyACM3", "303a", "1001", "SHOULDNOTPRINT");
        let found = scan_from(&by_id, &dev, &sys_tty).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, PortKind::PaperMonoUsb);
        assert_eq!(found[0].vid, Some(ESPRESSIF_VID));
        assert_eq!(found[0].pid, Some(ESPRESSIF_JTAG_PID));
        assert_eq!(found[0].tty_name.as_deref(), Some("ttyACM3"));
    }

    fn papermono_candidate(port: &str) -> Candidate {
        Candidate {
            by_id: Some(PathBuf::from(port)),
            tty_name: Some("ttyACM1".into()),
            usb_serial: Some("TESTUSB".into()),
            vid: Some(ESPRESSIF_VID),
            pid: Some(ESPRESSIF_JTAG_PID),
            product: Some("USB JTAG/serial debug unit".into()),
            kind: PortKind::PaperMonoUsb,
        }
    }

    #[test]
    fn inventory_hides_qinheng_unless_all_devices() {
        let paper = papermono_candidate("/tmp/by-id-paper");
        let other = Candidate {
            by_id: Some(PathBuf::from("/tmp/ch343")),
            tty_name: Some("ttyACM0".into()),
            usb_serial: Some("OTHER".into()),
            vid: Some(QINHENG_VID),
            pid: Some(QINHENG_PID),
            product: None,
            kind: PortKind::QinHengCh343,
        };
        let all = [paper, other];
        let listed = listed_for_inventory(&all, false);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].kind, PortKind::PaperMonoUsb);
        assert_eq!(listed_for_inventory(&all, true).len(), 2);
    }

    #[test]
    fn require_refuses_qinheng_before_open() {
        let c = Candidate {
            by_id: None,
            tty_name: Some("ttyACM0".into()),
            usb_serial: None,
            vid: Some(QINHENG_VID),
            pid: Some(QINHENG_PID),
            product: None,
            kind: PortKind::QinHengCh343,
        };
        assert!(matches!(
            require_papermono_usb_from("ttyACM0", std::slice::from_ref(&c), Path::new("/no-sys")),
            Err(Error::QinHengCh343)
        ));
    }

    #[test]
    fn redact_replaces_usb_serial_in_by_id() {
        let path =
            "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_AA:BB:CC:DD:EE:FF-if00";
        let shown = redact_identity_in_path(path, Some("AA:BB:CC:DD:EE:FF"));
        assert!(!shown.contains("AA:BB"));
        assert!(shown.contains("<redacted>"));
    }

    #[test]
    fn require_accepts_espressif() {
        let c = papermono_candidate("/tmp/by-id-paper");
        require_papermono_usb_from(
            "/tmp/by-id-paper",
            std::slice::from_ref(&c),
            Path::new("/no"),
        )
        .unwrap();
    }
}
