//! ST25R3916 error types.

use core::fmt;

/// Driver error type parameterized over the underlying bus error `E`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// Bus communication error.
    Bus(E),
    /// Operation or hardware timer timed out.
    Timeout,
    /// Crystal oscillator did not stabilize within expected cycle budget.
    OscillatorNotReady,
    /// Unexpected IC type byte read from identity register.
    WrongChipType(u8),
    /// Bit collision detected during anticollision or reception.
    Collision,
    /// FIFO underflow or incomplete frame read.
    FifoUnderflow,
    /// Invalid parameter or configuration values.
    InvalidParameter,
    /// CRC check failure on received frame.
    CrcError,
}

impl<E> From<E> for Error<E> {
    #[inline]
    fn from(err: E) -> Self {
        Self::Bus(err)
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bus(e) => write!(f, "ST25R3916 bus error: {e}"),
            Self::Timeout => write!(f, "ST25R3916 operation timed out"),
            Self::OscillatorNotReady => write!(f, "ST25R3916 oscillator not stable"),
            Self::WrongChipType(id) => write!(f, "ST25R3916 unexpected chip ID: {id:#04x}"),
            Self::Collision => write!(f, "ST25R3916 collision detected"),
            Self::FifoUnderflow => write!(f, "ST25R3916 FIFO underflow"),
            Self::InvalidParameter => write!(f, "ST25R3916 invalid parameter"),
            Self::CrcError => write!(f, "ST25R3916 CRC error"),
        }
    }
}
