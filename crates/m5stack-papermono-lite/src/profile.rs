//! Hardware model identification and board feature profile.
//!
//! Provides the [`BoardModel`] enum and helper predicates for distinguishing
//! between the standard PaperMono (`C153`) and PaperMono-Lite (`C153-Lite`).

/// Commercial hardware models for the M5Stack PaperMono product line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoardModel {
    /// Standard PaperMono (`C153`) with populated ST25R3916 NFC and Stamp LoRa-1262.
    PaperMono,
    /// PaperMono-Lite (`C153-Lite`) baseline with unpopulated NFC and LoRa.
    PaperMonoLite,
}

impl BoardModel {
    /// Official commercial Store SKU string (`C153` or `C153-Lite`).
    #[inline]
    #[must_use]
    pub const fn sku(&self) -> &'static str {
        match self {
            Self::PaperMono => "C153",
            Self::PaperMonoLite => "C153-Lite",
        }
    }

    /// Human-readable product name (`PaperMono` or `PaperMono-Lite`).
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PaperMono => "PaperMono",
            Self::PaperMonoLite => "PaperMono-Lite",
        }
    }

    /// Whether this board model populates the ST25R3916 NFC transceiver.
    #[inline]
    #[must_use]
    pub const fn has_nfc(&self) -> bool {
        matches!(self, Self::PaperMono)
    }

    /// Whether this board model populates the Stamp LoRa-1262 (SX1262) transceiver.
    #[inline]
    #[must_use]
    pub const fn has_lora(&self) -> bool {
        matches!(self, Self::PaperMono)
    }

    /// Whether this board model populates optional radio hardware (NFC and LoRa).
    #[inline]
    #[must_use]
    pub const fn has_radios(&self) -> bool {
        matches!(self, Self::PaperMono)
    }

    /// Decodes a raw integer identifier into a [`BoardModel`].
    ///
    /// Used for zero-overhead lock-free atomic storage across tasks.
    #[inline]
    #[must_use]
    pub const fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::PaperMonoLite),
            2 => Some(Self::PaperMono),
            _ => None,
        }
    }

    /// Encodes this [`BoardModel`] into a raw integer identifier.
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::PaperMonoLite => 1,
            Self::PaperMono => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_properties() {
        let full = BoardModel::PaperMono;
        assert_eq!(full.sku(), "C153");
        assert_eq!(full.name(), "PaperMono");
        assert!(full.has_nfc());
        assert!(full.has_lora());
        assert!(full.has_radios());
        assert_eq!(full.to_u8(), 2);
        assert_eq!(BoardModel::from_u8(2), Some(full));

        let lite = BoardModel::PaperMonoLite;
        assert_eq!(lite.sku(), "C153-Lite");
        assert_eq!(lite.name(), "PaperMono-Lite");
        assert!(!lite.has_nfc());
        assert!(!lite.has_lora());
        assert!(!lite.has_radios());
        assert_eq!(lite.to_u8(), 1);
        assert_eq!(BoardModel::from_u8(1), Some(lite));

        assert_eq!(BoardModel::from_u8(0), None);
        assert_eq!(BoardModel::from_u8(3), None);
    }
}
