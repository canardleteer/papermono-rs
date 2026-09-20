//! ISO/IEC 14443-4 (ISO-DEP) Transmission Protocol implementation.
//!
//! Citations:
//! - ISO/IEC 14443-4:2008 / 2018: "Identification cards — Contactless integrated circuit cards — Proximity cards — Part 4: Transmission protocol"
//! - ISO/IEC 7816-4: "Identification cards — Integrated circuit cards — Part 4: Organization, security and commands for interchange"

pub mod block;
pub mod error;
pub mod rats;
pub mod session;

pub use block::{IBlock, Pcb, RBlock, SBlock, SBlockKind};
pub use error::{IsoDepError, IsoDepErrorKind};
pub use rats::{build_rats, fsi_to_bytes, Ats, FSDI_256_BYTES, RATS_CMD_START};
pub use session::{IsoDepSession, DEFAULT_FWT_POLL_ITERATIONS, MAX_ISODEP_FRAME};
