//! ISO/IEC 14443-4 transmission protocol error definitions.
//!
//! Citations:
//! - ISO/IEC 14443-4:2008 / 2018: "Identification cards — Contactless integrated circuit cards — Proximity cards — Part 4: Transmission protocol"

/// ISO/IEC 14443-4 error kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoDepErrorKind {
    /// RATS request was rejected or no Answer To Select (ATS) received.
    RatsFailed,
    /// Invalid or malformed ATS received.
    InvalidAts,
    /// Timeout waiting for PICC response (exceeded Frame Waiting Time).
    Timeout,
    /// Protocol error (invalid PCB, unsupported block, sequence number mismatch).
    ProtocolError,
    /// Response buffer provided by caller has insufficient capacity.
    BufferTooSmall,
    /// S(DESELECT) command failed or card was deselected.
    Deselected,
    /// Chaining protocol error (e.g. invalid R(ACK) block number).
    ChainingError,
}

/// ISO/IEC 14443-4 error wrapping bus transport errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoDepError<E> {
    /// Low-level bus communication failure (e.g. I2C error).
    Bus(E),
    /// ISO-DEP protocol-level failure.
    Protocol(IsoDepErrorKind),
}

impl<E> From<IsoDepErrorKind> for IsoDepError<E> {
    #[inline]
    fn from(kind: IsoDepErrorKind) -> Self {
        Self::Protocol(kind)
    }
}
