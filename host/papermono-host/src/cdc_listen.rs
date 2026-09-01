//! CDC ACM listen that never opens the kernel TTY.
//!
//! Linux `cdc-acm` can assert DTR/RTS in `acm_port_activate`. On
//! ESP32-S3 USB-Serial/JTAG that may reset the chip. This path claims
//! the USB interfaces and leaves the modem lines deasserted.

use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use nusb::descriptors::TransferType;
use nusb::transfer::{Bulk, ControlOut, ControlType, Direction, In, Recipient};
use nusb::MaybeFuture;

use crate::detect::{usb_device_key_for_port, PortKind, ESPRESSIF_JTAG_PID, ESPRESSIF_VID};
use crate::Error;

const SET_LINE_CODING: u8 = 0x20;
const SET_CONTROL_LINE_STATE: u8 = 0x22;
const CDC_COMM: u8 = 0x02;
const CDC_ACM: u8 = 0x02;
const CDC_DATA: u8 = 0x0A;

const USB_TIMEOUT: Duration = Duration::from_millis(250);

static INTERRUPT: AtomicBool = AtomicBool::new(false);
static INTERRUPT_HANDLER: OnceLock<()> = OnceLock::new();

/// Catch SIGINT/SIGTERM so [`CdcListen`] Drop can reattach `cdc-acm`.
pub fn catch_interrupt() {
    INTERRUPT_HANDLER.get_or_init(|| {
        let _ = ctrlc::set_handler(|| {
            INTERRUPT.store(true, Ordering::SeqCst);
        });
    });
}

/// True after Ctrl-C / SIGTERM. Listen loops should break so Drop runs.
#[must_use]
pub fn interrupt_requested() -> bool {
    INTERRUPT.load(Ordering::Relaxed)
}

/// Open the Espressif USB-Serial/JTAG as USB CDC without pulsing DTR.
pub struct CdcListen {
    reader: Option<nusb::io::EndpointRead<Bulk>>,
    data: Option<nusb::Interface>,
    comm: Option<nusb::Interface>,
    device: nusb::Device,
    comm_num: u8,
    data_num: u8,
}

impl CdcListen {
    /// Claim the USB device that backs `port`. Does not open the ACM node.
    pub fn open(port: &str) -> Result<Self, Error> {
        catch_interrupt();
        crate::detect::require_papermono_usb(port)?;
        let key = usb_device_key_for_port(port)?;
        if key.vid != ESPRESSIF_VID || key.pid != ESPRESSIF_JTAG_PID {
            return Err(Error::NotPaperMonoUsb {
                vid: Some(key.vid),
                pid: Some(key.pid),
            });
        }
        let info = nusb::list_devices()
            .wait()
            .map_err(|error| Error::Device(format!("USB list failed: {error}")))?
            .find(|dev| {
                dev.busnum() == key.busnum
                    && dev.device_address() == key.devnum
                    && dev.vendor_id() == key.vid
                    && dev.product_id() == key.pid
            })
            .ok_or_else(|| {
                Error::Device(format!(
                    "USB device bus {} addr {} is not visible to usbfs",
                    key.busnum, key.devnum
                ))
            })?;
        let device = info.open().wait().map_err(map_usb_open)?;
        let config = device
            .active_configuration()
            .map_err(|error| Error::Device(format!("USB configuration: {error}")))?;
        let layout = find_cdc_layout(&config)?;
        let comm = device
            .detach_and_claim_interface(layout.comm)
            .wait()
            .map_err(map_usb_open)?;
        let data = device
            .detach_and_claim_interface(layout.data)
            .wait()
            .map_err(map_usb_open)?;
        set_listen_coding(&comm, layout.comm)?;
        let mut reader = data
            .endpoint::<Bulk, In>(layout.bulk_in)
            .map_err(|error| Error::Device(format!("USB bulk-in: {error}")))?
            .reader(4096);
        reader.set_read_timeout(USB_TIMEOUT);
        log::info!(
            "CDC listen bus {} addr {} at 115200 (no ACM TTY, modem lines off)",
            key.busnum,
            key.devnum
        );
        Ok(Self {
            reader: Some(reader),
            data: Some(data),
            comm: Some(comm),
            device,
            comm_num: layout.comm,
            data_num: layout.data,
        })
    }
}

impl Read for CdcListen {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "CDC listen closed"))?
            .read(buf)
    }
}

impl Drop for CdcListen {
    fn drop(&mut self) {
        self.reader = None;
        self.data = None;
        self.comm = None;
        reattach(&self.device, self.data_num, "data");
        reattach(&self.device, self.comm_num, "comm");
    }
}

fn reattach(device: &nusb::Device, iface: u8, name: &str) {
    if let Err(error) = device.attach_kernel_driver(iface) {
        log::warn!("reattach cdc-acm {name} (if {iface}): {error}");
    }
}

/// True when inventory sees an Espressif TTY again (kernel driver bound).
#[must_use]
pub fn wait_for_kernel_tty(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if crate::detect::scan().ok().is_some_and(|cands| {
            cands
                .iter()
                .any(|cand| cand.kind == PortKind::PaperMonoUsb && cand.preferred_port().is_some())
        }) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

struct CdcLayout {
    comm: u8,
    data: u8,
    bulk_in: u8,
}

/// CDC ACM comm + data bulk-in only.
///
/// ESP32-S3 USB-Serial/JTAG also has a vendor JTAG interface with
/// its own bulk-in (`0x83`) after CDC data (`0x81`). Taking the last
/// bulk-in on the device then opening it on the data interface fails
/// with "specified endpoint does not exist on this interface".
fn find_cdc_layout(
    config: &nusb::descriptors::ConfigurationDescriptor<'_>,
) -> Result<CdcLayout, Error> {
    let mut comm = None;
    let mut data = None;
    let mut bulk_in = None;
    for alt in config.interface_alt_settings() {
        if alt.alternate_setting() != 0 {
            continue;
        }
        if alt.class() == CDC_COMM && alt.subclass() == CDC_ACM {
            comm = Some(alt.interface_number());
            continue;
        }
        if alt.class() != CDC_DATA {
            continue;
        }
        data = Some(alt.interface_number());
        for ep in alt.endpoints() {
            if ep.transfer_type() == TransferType::Bulk && ep.direction() == Direction::In {
                bulk_in = Some(ep.address());
            }
        }
    }
    match (comm, data, bulk_in) {
        (Some(comm), Some(data), Some(bulk_in)) => Ok(CdcLayout {
            comm,
            data,
            bulk_in,
        }),
        _ => Err(Error::Device(
            "USB descriptors have no CDC ACM + bulk-in pair".into(),
        )),
    }
}

fn set_listen_coding(comm: &nusb::Interface, comm_num: u8) -> Result<(), Error> {
    let mut coding = [0u8; 7];
    coding[..4].copy_from_slice(&crate::monitor::MONITOR_BAUD.to_le_bytes());
    coding[4] = 0;
    coding[5] = 0;
    coding[6] = 8;
    comm.control_out(
        ControlOut {
            control_type: ControlType::Class,
            recipient: Recipient::Interface,
            request: SET_LINE_CODING,
            value: 0,
            index: u16::from(comm_num),
            data: &coding,
        },
        USB_TIMEOUT,
    )
    .wait()
    .map_err(|error| Error::Device(format!("SET_LINE_CODING: {error}")))?;
    comm.control_out(
        ControlOut {
            control_type: ControlType::Class,
            recipient: Recipient::Interface,
            request: SET_CONTROL_LINE_STATE,
            value: 0,
            index: u16::from(comm_num),
            data: &[],
        },
        USB_TIMEOUT,
    )
    .wait()
    .map_err(|error| Error::Device(format!("SET_CONTROL_LINE_STATE: {error}")))?;
    Ok(())
}

fn map_usb_open(error: nusb::Error) -> Error {
    let text = error.to_string();
    if matches!(
        error.kind(),
        nusb::ErrorKind::PermissionDenied | nusb::ErrorKind::Busy
    ) || text.contains("Permission denied")
        || text.contains("Access denied")
    {
        return Error::Device(format!(
            "{text}. `monitor` claims usbfs so it does not open the ACM TTY \
             (cdc-acm can assert DTR). Copy \
             host/papermono-host/udev/99-papermono-usb.rules to \
             /etc/udev/rules.d/, run `sudo udevadm control --reload-rules` \
             and `sudo udevadm trigger`, then unplug and replug. \
             Check `ls -l /dev/bus/usb/BBB/DDD` from `lsusb` Bus/Device \
             (zero-pad to 3 digits); expect GROUP dialout MODE 0660."
        ));
    }
    Error::Device(format!("USB open failed: {text}"))
}

#[cfg(test)]
mod tests {
    use super::find_cdc_layout;
    use nusb::descriptors::ConfigurationDescriptor;

    fn tiny_cdc_config() -> Vec<u8> {
        vec![
            9, 2, 41, 0, 2, 1, 0, 0x80, 50, 9, 4, 0, 0, 1, 0x02, 0x02, 0x01, 0, 7, 5, 0x83, 0x03,
            8, 0, 10, 9, 4, 1, 0, 1, 0x0A, 0x00, 0x00, 0, 7, 5, 0x81, 0x02, 64, 0, 0,
        ]
    }

    /// ESP32-S3 USB-Serial/JTAG: CDC plus a later vendor JTAG bulk-in.
    fn esp32s3_jtag_serial_config() -> Vec<u8> {
        vec![
            9, 2, 71, 0, 3, 1, 0, 0x80, 50, 9, 4, 0, 0, 1, 0x02, 0x02, 0x01, 0, 7, 5, 0x82, 0x03,
            8, 0, 10, 9, 4, 1, 0, 2, 0x0A, 0x00, 0x00, 0, 7, 5, 0x01, 0x02, 64, 0, 0, 7, 5, 0x81,
            0x02, 64, 0, 0, 9, 4, 2, 0, 2, 0xFF, 0xFF, 0x00, 0, 7, 5, 0x02, 0x02, 64, 0, 0, 7, 5,
            0x83, 0x02, 64, 0, 0,
        ]
    }

    #[test]
    fn interrupt_starts_clear() {
        assert!(!super::interrupt_requested());
    }

    #[test]
    fn layout_finds_acm_and_bulk_in() {
        let bytes = tiny_cdc_config();
        let config = ConfigurationDescriptor::new(&bytes).expect("config");
        let layout = find_cdc_layout(&config).expect("cdc");
        assert_eq!(layout.comm, 0);
        assert_eq!(layout.data, 1);
        assert_eq!(layout.bulk_in, 0x81);
    }

    #[test]
    fn layout_ignores_vendor_jtag_bulk_in() {
        let bytes = esp32s3_jtag_serial_config();
        let config = ConfigurationDescriptor::new(&bytes).expect("config");
        let layout = find_cdc_layout(&config).expect("cdc");
        assert_eq!(layout.comm, 0);
        assert_eq!(layout.data, 1);
        assert_eq!(layout.bulk_in, 0x81);
    }
}
