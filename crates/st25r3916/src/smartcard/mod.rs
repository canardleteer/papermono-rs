//! Smart card application discovery and protocol drivers over ISO/IEC 14443-4 (ISO-DEP).
//!
//! Citations:
//! - FIDO Alliance: "Client to Authenticator Protocol (CTAP) Implementation Draft"
//! - NIST Special Publication 800-73-4: "Interfaces for Personal Identity Verification"
//! - OpenPGP application on ISO/IEC 7816-4 compatible smart cards (v3.4.1)

pub mod fido;
pub mod openpgp;
pub mod piv;

use embedded_hal::i2c::I2c;

pub use fido::{probe_fido, FidoCardInfo, AID_FIDO};
pub use openpgp::{manufacturer_name, probe_openpgp, OpenPgpCardInfo, AID_OPENPGP};
pub use piv::{probe_piv, PivCardInfo, AID_PIV};

use crate::isodep::{Ats, IsoDepError, IsoDepSession};
use crate::St25r3916;

/// Discovered capabilities and profiles on a contactless YubiKey authenticator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YubiKeyProfiles {
    /// FIDO2 / CTAP / U2F profile, if present.
    pub fido: Option<FidoCardInfo>,
    /// NIST SP 800-73-4 PIV profile, if present.
    pub piv: Option<PivCardInfo>,
    /// OpenPGP smart card profile, if present.
    pub openpgp: Option<OpenPgpCardInfo>,
    /// ATS received during activation.
    pub ats: Ats,
    /// Raw status word returned by FIDO SELECT probe.
    pub fido_sw: u16,
    /// Raw status word returned by PIV SELECT probe.
    pub piv_sw: u16,
    /// Raw status word returned by OpenPGP SELECT probe.
    pub openpgp_sw: u16,
}

/// High-level detected smart card application over ISO-DEP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartCardApp {
    /// FIDO2 / CTAP / U2F security key.
    Fido(FidoCardInfo),
    /// Personal Identity Verification (NIST SP 800-73-4) credential.
    Piv(PivCardInfo),
    /// OpenPGP smart card (e.g. YubiKey OpenPGP applet, Nitrokey).
    OpenPgp(OpenPgpCardInfo),
    /// YubiKey multi-application authenticator (with all probed profiles).
    YubiKey(YubiKeyProfiles),
    /// Generic or unrecognized ISO-DEP (Type 4A) smart card.
    Generic(Ats),
}

/// Probes an activated ISO-DEP transponder to discover supported smart card applications.
///
/// Sequentially checks for:
/// 1. FIDO CTAP (U2F / FIDO2)
/// 2. NIST SP 800-73-4 PIV
/// 3. OpenPGP Card (v3.4)
///
/// If ATS historical bytes indicate a YubiKey or multiple security profiles are present,
/// returns `SmartCardApp::YubiKey(YubiKeyProfiles)`.
/// If a single specific profile matches, returns `Fido`, `Piv`, or `OpenPgp`.
/// Otherwise returns `SmartCardApp::Generic(ats)`.
pub fn probe_smart_card<I2C: I2c>(
    st: &mut St25r3916<I2C>,
    ats: &Ats,
) -> Result<SmartCardApp, IsoDepError<I2C::Error>> {
    let mut session = IsoDepSession {
        fsc: ats.fsc,
        fsd: 256,
        cid: None,
        fwi: ats.fwi(),
        pcd_block_num: 0,
        picc_block_num: 0,
    };

    let (fido, fido_sw) = fido::probe_fido_detail(&mut session, st);
    let (piv, piv_sw) = piv::probe_piv_detail(&mut session, st);
    let (openpgp, openpgp_sw) = openpgp::probe_openpgp_detail(&mut session, st);

    let _ = session.deselect(st);

    if ats.is_yubikey() || (fido.is_some() && (piv.is_some() || openpgp.is_some())) {
        return Ok(SmartCardApp::YubiKey(YubiKeyProfiles {
            fido,
            piv,
            openpgp,
            ats: ats.clone(),
            fido_sw,
            piv_sw,
            openpgp_sw,
        }));
    }

    if let Some(f) = fido {
        return Ok(SmartCardApp::Fido(f));
    }
    if let Some(p) = piv {
        return Ok(SmartCardApp::Piv(p));
    }
    if let Some(o) = openpgp {
        return Ok(SmartCardApp::OpenPgp(o));
    }

    Ok(SmartCardApp::Generic(ats.clone()))
}
