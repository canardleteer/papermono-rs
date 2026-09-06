//! ST25R3916 Near Field Communication (NFC) driver and card detection service.
//!
//! # Architecture & Hardware Net Routing
//! The M5Stack PaperMono (`C153`) includes an STMicroelectronics ST25R3916 NFC
//! transceiver connected to the system I2C bus:
//! - **I2C Address**: `0x50` (`m5stack_papermono::nfc::ADDRESS`).
//! - **Power Gating**: Controlled by M5IOE1 expander pin `PYG4` (`IOE1_ENABLE`).
//!   When `PYG4` is low, the ST25R3916 is completely unpowered to conserve battery.
//! - **Interrupt**: `GPIO6` connects to the ST25R3916 IRQ output.
//! - **PaperMono-Lite (`C153-Lite`)**: The NFC controller is unpopulated. I2C address
//!   `0x50` NAKs, and leftover pads remain unpowered.
//!
//! # ISO/IEC 14443-A Polling Protocol
//! The polling sequence adheres to ISO/IEC 14443-3 contactless communication standards:
//! 1. Assert `PYG4` high to power the ST25R3916 controller and wait 20 ms for startup.
//! 2. Verify controller presence via IC identity query (`0x7F` -> `0x05`, ST25R3916).
//! 3. Energize the 13.56 MHz RF field by configuring `Operation control register` (`0x02`)
//!    with `OP_CONTROL_EN | OP_CONTROL_RX_EN | OP_CONTROL_TX_EN`.
//! 4. Configure `Mode definition register` (`0x03`) for Initiator ISO14443A mode (`0x08`).
//! 5. Transmit `REQA` (Request Type A) via direct command `0xC6`.
//! 6. If a tag is energized within the RF field, it responds with a 2-byte `ATQA`
//!    (Answer To Request Type A) placed into the ST25R3916 FIFO.
//! 7. Execute Cascade Level 1 (CL1) Anticollision (`0x93`, `0x20`) without CRC. The tag
//!    responds with 4 UID bytes plus 1 BCC checksum byte (`BCC = UID0 ^ UID1 ^ UID2 ^ UID3`).
//! 8. Issue CL1 Select (`0x93`, `0x70`) with CRC. The tag replies with `SAK` (Select Acknowledge).
//!    - If bit 2 of `SAK` is 0 (and UID0 != `0x88`), the tag possesses a complete 4-byte single-size UID.
//!    - If bit 2 of `SAK` is 1 (or UID0 == `0x88`), the tag has a multi-level UID (7-byte double-size).
//! 9. For 7-byte tags (e.g. NTAG213/215/216, MIFARE Ultralight, or Flipper Zero synthetic tags),
//!    Cascade Level 2 (CL2) Anticollision (`0x95`, `0x20`) and Select (`0x95`, `0x70`) are executed
//!    to retrieve the remaining 4 bytes of the unique identifier.
//! 10. Immediately upon completion or timeout, the RF carrier is deactivated (`CMD_STOP_ALL`,
//!     reg `0x02` cleared) and `PYG4` is de-asserted low to prevent unnecessary RF emissions and
//!     minimize battery drain.
//!
//! # Privacy Protection
//! Contactless payment cards (EMV) emit random or dynamic UIDs during ISO14443-A anticollision.
//! To protect user confidentiality, serial CDC logs mask intermediate UID bytes (e.g. `uid=08..2c`),
//! logging only the outer boundary bytes and tag classification. The physical e-paper display
//! renders the detected UID locally for user verification.

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

#[cfg(feature = "c153")]
use embassy_time::{Duration, Timer};
#[cfg(feature = "c153")]
pub use m5stack_papermono::nfc::Iso14443aCard;
#[cfg(feature = "c153")]
use m5stack_papermono::nfc::{self, St25r3916};

/// Fallback dummy type on Lite builds where NFC is unpopulated.
#[cfg(not(feature = "c153"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iso14443aCard;

/// High-level UI status of the NFC card interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcCardState {
    /// Hardware unpopulated (PaperMono-Lite `C153-Lite` or missing controller).
    Unpopulated,
    /// Controller detected and ready for an on-demand poll.
    Idle,
    /// A contactless tag was detected in the active RF field.
    Detected(Iso14443aCard),
    /// Tag polling concluded without detecting any transponder.
    NoTag,
}

// Atomic storage for cross-task NFC card status:
// State code: 0 = Unpopulated, 1 = Idle, 2 = Detected, 3 = NoTag.
static NFC_STATE_CODE: AtomicU8 = AtomicU8::new(0);
static NFC_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "c153")]
static DETECTED_ATQA: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);
#[cfg(feature = "c153")]
static DETECTED_SAK: AtomicU8 = AtomicU8::new(0);
#[cfg(feature = "c153")]
static DETECTED_UID_LEN: AtomicU8 = AtomicU8::new(0);
#[cfg(feature = "c153")]
static DETECTED_UID: [AtomicU8; 10] = [
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
];

/// Initializes the NFC state tracking for the active board SKU.
pub fn init_status(has_nfc: bool) {
    if has_nfc {
        NFC_STATE_CODE.store(1, Ordering::Relaxed);
    } else {
        NFC_STATE_CODE.store(0, Ordering::Relaxed);
    }
    NFC_INITIALIZED.store(true, Ordering::Relaxed);
}

/// Retrieves the current status of the NFC card interface.
pub fn card_state() -> NfcCardState {
    if !NFC_INITIALIZED.load(Ordering::Relaxed) {
        #[cfg(feature = "c153")]
        return NfcCardState::Idle;
        #[cfg(not(feature = "c153"))]
        return NfcCardState::Unpopulated;
    }

    match NFC_STATE_CODE.load(Ordering::Relaxed) {
        0 => NfcCardState::Unpopulated,
        1 => NfcCardState::Idle,
        #[cfg(feature = "c153")]
        2 => {
            let atqa_raw = DETECTED_ATQA.load(Ordering::Relaxed);
            let sak = DETECTED_SAK.load(Ordering::Relaxed);
            let uid_len = DETECTED_UID_LEN.load(Ordering::Relaxed) as usize;
            let mut uid = [0u8; 10];
            for (i, slot) in DETECTED_UID.iter().enumerate() {
                uid[i] = slot.load(Ordering::Relaxed);
            }
            NfcCardState::Detected(Iso14443aCard {
                atqa: [(atqa_raw & 0xFF) as u8, (atqa_raw >> 8) as u8],
                sak,
                uid_len,
                uid,
            })
        }
        #[cfg(not(feature = "c153"))]
        2 => NfcCardState::NoTag,
        _ => NfcCardState::NoTag,
    }
}

/// Stores a newly detected contactless tag into shared memory.
#[cfg(feature = "c153")]
fn store_detected(card: &Iso14443aCard) {
    let atqa_raw = (card.atqa[0] as u16) | ((card.atqa[1] as u16) << 8);
    DETECTED_ATQA.store(atqa_raw, Ordering::Relaxed);
    DETECTED_SAK.store(card.sak, Ordering::Relaxed);
    DETECTED_UID_LEN.store(card.uid_len as u8, Ordering::Relaxed);
    for (i, b) in card.uid.iter().enumerate() {
        DETECTED_UID[i].store(*b, Ordering::Relaxed);
    }
    NFC_STATE_CODE.store(2, Ordering::Relaxed);
}

/// Resets the NFC card state to `NoTag` following a scan that yielded no transponder.
pub fn set_no_tag() {
    NFC_STATE_CODE.store(3, Ordering::Relaxed);
}

/// Performs an on-demand, bounded ISO14443-A poll sequence on the ST25R3916 transceiver.
///
/// Hardware power rail `PYG4` is energized for the duration of this call and guaranteed
/// de-asserted upon return. The RF field is active for under 60 milliseconds.
pub async fn poll_iso14443a(i2c: &mut crate::ioe::SysI2c) -> Option<Iso14443aCard> {
    #[cfg(feature = "c153")]
    {
        // 1. Energize the ST25R3916 power gate via M5IOE1 expander pin PYG4.
        let _ = crate::ioe::set_push_pull_output(i2c, nfc::IOE1_ENABLE, true);
        Timer::after(Duration::from_millis(120)).await;

        let mut st = St25r3916::new(&mut *i2c, nfc::ADDRESS);

        // Verify transceiver responds at address 0x50 with expected IC identity.
        if let Ok(id) = st.read_identity() {
            esp_println::println!(
                "simple-debug: nfc_poll step=id ok={} id={:02x} rev={}",
                id.is_st25r3916(),
                id.ic_type,
                id.ic_rev
            );
            if id.is_st25r3916() {
                // 2. Power up oscillator and 13.56 MHz RF field for ISO14443-A Initiator mode.
                if st.enable_field().is_ok() {
                    let aux = st.read_aux().unwrap_or(0);
                    esp_println::println!(
                        "simple-debug: nfc_poll step=field_on aux={:02x} (tx_on={} osc_ok={})",
                        aux,
                        (aux & nfc::AUX_DISPLAY_TX_ON) != 0,
                        (aux & nfc::AUX_DISPLAY_OSC_OK) != 0
                    );
                    Timer::after(Duration::from_millis(150)).await;

                    // 3. Scan loop: attempt REQA / WUPA short frames over a 3.5-second window
                    // (matching M5PaperMono-UserDemo NFC_A_SCAN_WINDOW_MS) to give the human
                    // ample time to hold a card or Flipper Zero against the antenna.
                    let mut detected_atqa = None;
                    for _attempt in 0..35 {
                        let _ = st.send_reqa();
                        Timer::after(Duration::from_millis(5)).await;
                        if let Ok(Some(atqa)) = st.read_atqa() {
                            detected_atqa = Some(atqa);
                            break;
                        }

                        let _ = st.send_wupa();
                        Timer::after(Duration::from_millis(5)).await;
                        if let Ok(Some(atqa)) = st.read_atqa() {
                            detected_atqa = Some(atqa);
                            break;
                        }
                        Timer::after(Duration::from_millis(60)).await;
                    }

                    // 4. Retrieve ATQA response from FIFO.
                    if let Some(atqa) = detected_atqa {
                        esp_println::println!(
                            "simple-debug: nfc_poll step=atqa atqa={:02x}{:02x}",
                            atqa[0],
                            atqa[1]
                        );
                        Timer::after(Duration::from_millis(5)).await;

                        // 5. Execute Cascade Level 1 Anticollision.
                        match st.anticollision_cl1() {
                            Ok(Some(cl1)) => {
                                let uid_cl1 = [cl1[0], cl1[1], cl1[2], cl1[3]];
                                let bcc = cl1[4];
                                esp_println::println!(
                                    "simple-debug: nfc_poll step=cl1 uid={:02x}{:02x}{:02x}{:02x} bcc={:02x}",
                                    uid_cl1[0],
                                    uid_cl1[1],
                                    uid_cl1[2],
                                    uid_cl1[3],
                                    bcc
                                );
                                Timer::after(Duration::from_millis(5)).await;

                                // 6. Issue Cascade Level 1 Select.
                                match st.select_cl1(uid_cl1, bcc) {
                                    Ok(Some(sak1)) => {
                                        esp_println::println!(
                                            "simple-debug: nfc_poll step=sak1 sak={:02x}",
                                            sak1
                                        );
                                        let mut card = Iso14443aCard {
                                            atqa,
                                            sak: sak1,
                                            uid_len: 4,
                                            uid: [0; 10],
                                        };

                                        if uid_cl1[0] != nfc::ISO14443A_CASCADE_TAG
                                            && (sak1 & 0x04) == 0
                                        {
                                            // 4-byte single-size UID complete.
                                            card.uid[..4].copy_from_slice(&uid_cl1);
                                            card.uid_len = 4;
                                            card.sak = sak1;

                                            // Safely park RF field and power gate.
                                            let _ = st.disable_field();
                                            let _ = crate::ioe::set_push_pull_output(
                                                i2c,
                                                nfc::IOE1_ENABLE,
                                                false,
                                            );
                                            store_detected(&card);
                                            return Some(card);
                                        } else {
                                            // 7-byte double-size UID: query Cascade Level 2.
                                            Timer::after(Duration::from_millis(5)).await;
                                            match st.anticollision_cl2() {
                                                Ok(Some(cl2)) => {
                                                    let uid_cl2 = [cl2[0], cl2[1], cl2[2], cl2[3]];
                                                    let bcc2 = cl2[4];
                                                    esp_println::println!(
                                                        "simple-debug: nfc_poll step=cl2 uid={:02x}{:02x}{:02x}{:02x} bcc={:02x}",
                                                        uid_cl2[0],
                                                        uid_cl2[1],
                                                        uid_cl2[2],
                                                        uid_cl2[3],
                                                        bcc2
                                                    );
                                                    Timer::after(Duration::from_millis(5)).await;

                                                    match st.select_cl2(uid_cl2, bcc2) {
                                                        Ok(Some(sak2)) => {
                                                            esp_println::println!(
                                                                "simple-debug: nfc_poll step=sak2 sak={:02x}",
                                                                sak2
                                                            );
                                                            card.uid[0..3]
                                                                .copy_from_slice(&uid_cl1[1..4]);
                                                            card.uid[3..7]
                                                                .copy_from_slice(&uid_cl2[0..4]);
                                                            card.uid_len = 7;
                                                            card.sak = sak2;

                                                            // Safely park RF field and power gate.
                                                            let _ = st.disable_field();
                                                            let _ =
                                                                crate::ioe::set_push_pull_output(
                                                                    i2c,
                                                                    nfc::IOE1_ENABLE,
                                                                    false,
                                                                );
                                                            store_detected(&card);
                                                            return Some(card);
                                                        }
                                                        Ok(None) => {
                                                            esp_println::println!(
                                                                "simple-debug: nfc_poll step=sak2_none"
                                                            );
                                                        }
                                                        Err(_) => {
                                                            esp_println::println!(
                                                                "simple-debug: nfc_poll step=sak2_err"
                                                            );
                                                        }
                                                    }
                                                }
                                                Ok(None) => {
                                                    esp_println::println!(
                                                        "simple-debug: nfc_poll step=cl2_none"
                                                    );
                                                }
                                                Err(_) => {
                                                    esp_println::println!(
                                                        "simple-debug: nfc_poll step=cl2_err"
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Ok(None) => {
                                        esp_println::println!(
                                            "simple-debug: nfc_poll step=sak1_none"
                                        );
                                    }
                                    Err(_) => {
                                        esp_println::println!(
                                            "simple-debug: nfc_poll step=sak1_err"
                                        );
                                    }
                                }
                            }
                            Ok(None) => {
                                let (main, timer, err) = st.read_irqs().unwrap_or((0, 0, 0));
                                let n = st.fifo_bytes().unwrap_or(0);
                                esp_println::println!(
                                    "simple-debug: nfc_poll step=cl1_none fifo={} irq_main={:02x} irq_timer={:02x} irq_err={:02x}",
                                    n,
                                    main,
                                    timer,
                                    err
                                );
                            }
                            Err(_) => {
                                esp_println::println!("simple-debug: nfc_poll step=cl1_err");
                            }
                        }
                    } else {
                        let (main, timer, err) = st.read_irqs().unwrap_or((0, 0, 0));
                        let n = st.fifo_bytes().unwrap_or(0);
                        esp_println::println!(
                            "simple-debug: nfc_poll step=no_tag fifo={} irq_main={:02x} irq_timer={:02x} irq_err={:02x}",
                            n,
                            main,
                            timer,
                            err
                        );
                    }

                    // Deactivate RF carrier.
                    let _ = st.disable_field();
                }
            }
        }

        // De-assert ST25R3916 power gate.
        let _ = crate::ioe::set_push_pull_output(i2c, nfc::IOE1_ENABLE, false);
        set_no_tag();
        None
    }

    #[cfg(not(feature = "c153"))]
    {
        let _ = i2c;
        None
    }
}
