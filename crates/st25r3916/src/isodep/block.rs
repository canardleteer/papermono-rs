//! ISO/IEC 14443-4 Section 7.1: Half-duplex block protocol framing and PCB encoding.
//!
//! Citations:
//! - ISO/IEC 14443-4:2008 / 2018:
//!   - Section 7.1: "Block format"
//!   - Section 7.1.1.1: "Protocol control byte field"
//!   - Figure 15: "Coding of I-block PCB"
//!   - Figure 16: "Coding of R-block PCB"
//!   - Figure 17: "Coding of S-block PCB"

use super::error::IsoDepErrorKind;

/// I-block PCB base mask (b8=0, b7=0, b2=1).
pub const PCB_I_BLOCK_BASE: u8 = 0x02;

/// I-block chaining (M) bit mask (b5).
pub const PCB_I_CHAINING_MASK: u8 = 0x10;

/// I-block CID present bit mask (b4).
pub const PCB_I_CID_MASK: u8 = 0x08;

/// I-block NAD present bit mask (b3).
pub const PCB_I_NAD_MASK: u8 = 0x04;

/// R-block PCB base mask (b8=1, b7=0, b6=1, b2=1).
pub const PCB_R_BLOCK_BASE: u8 = 0xA2;

/// R-block NAK bit mask (b5). 0 = ACK, 1 = NAK.
pub const PCB_R_NAK_MASK: u8 = 0x10;

/// R-block CID present bit mask (b4).
pub const PCB_R_CID_MASK: u8 = 0x08;

/// S-block PCB base mask (b8=1, b7=1, b2=1, b1=0).
pub const PCB_S_BLOCK_BASE: u8 = 0xC2;

/// S-block WTX type mask (b6=1, b5=1).
pub const PCB_S_WTX_MASK: u8 = 0x30;

/// S-block CID present bit mask (b4).
pub const PCB_S_CID_MASK: u8 = 0x08;

/// S-block supervisory control kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SBlockKind {
    /// DESELECT command to release PICC from ISO-DEP active state.
    Deselect,
    /// Waiting Time Extension (WTX) request/response.
    Wtx,
}

/// Information block (I-block) carrying application protocol data units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IBlock {
    /// Block sequence number (0 or 1).
    pub block_num: u8,
    /// Chaining indicator: `true` if subsequent chained blocks follow.
    pub chaining: bool,
    /// Card Identifier present flag.
    pub has_cid: bool,
    /// Node Address present flag.
    pub has_nad: bool,
}

impl IBlock {
    /// Encodes this I-block configuration into a Protocol Control Byte.
    #[inline]
    #[must_use]
    pub const fn encode(&self) -> u8 {
        let mut pcb = PCB_I_BLOCK_BASE | (self.block_num & 0x01);
        if self.chaining {
            pcb |= PCB_I_CHAINING_MASK;
        }
        if self.has_cid {
            pcb |= PCB_I_CID_MASK;
        }
        if self.has_nad {
            pcb |= PCB_I_NAD_MASK;
        }
        pcb
    }
}

/// Receive ready / acknowledgement block (R-block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RBlock {
    /// Expected next I-block sequence number (0 or 1).
    pub block_num: u8,
    /// `true` for positive acknowledgement (ACK), `false` for negative (NAK).
    pub ack: bool,
    /// Card Identifier present flag.
    pub has_cid: bool,
}

impl RBlock {
    /// Encodes this R-block into a Protocol Control Byte.
    #[inline]
    #[must_use]
    pub const fn encode(&self) -> u8 {
        let mut pcb = PCB_R_BLOCK_BASE | (self.block_num & 0x01);
        if !self.ack {
            pcb |= PCB_R_NAK_MASK;
        }
        if self.has_cid {
            pcb |= PCB_R_CID_MASK;
        }
        pcb
    }
}

/// Supervisory control block (S-block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SBlock {
    /// Supervisory command kind (DESELECT or WTX).
    pub kind: SBlockKind,
    /// Card Identifier present flag.
    pub has_cid: bool,
}

impl SBlock {
    /// Encodes this S-block into a Protocol Control Byte.
    #[inline]
    #[must_use]
    pub const fn encode(&self) -> u8 {
        let mut pcb = PCB_S_BLOCK_BASE;
        match self.kind {
            SBlockKind::Deselect => {}
            SBlockKind::Wtx => pcb |= PCB_S_WTX_MASK,
        }
        if self.has_cid {
            pcb |= PCB_S_CID_MASK;
        }
        pcb
    }
}

/// Decoded Protocol Control Byte (PCB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pcb {
    /// Application layer Information block.
    I(IBlock),
    /// Acknowledgement / Receive ready block.
    R(RBlock),
    /// Supervisory block.
    S(SBlock),
}

impl Pcb {
    /// Parses a raw Protocol Control Byte.
    pub fn parse(byte: u8) -> Result<Self, IsoDepErrorKind> {
        match byte & 0xC0 {
            0x00 => {
                // I-block: b8=0, b7=0, b6 must be 0, b2 must be 1
                if (byte & 0x40) != 0 || (byte & 0x02) == 0 {
                    return Err(IsoDepErrorKind::ProtocolError);
                }
                Ok(Self::I(IBlock {
                    block_num: byte & 0x01,
                    chaining: (byte & PCB_I_CHAINING_MASK) != 0,
                    has_cid: (byte & PCB_I_CID_MASK) != 0,
                    has_nad: (byte & PCB_I_NAD_MASK) != 0,
                }))
            }
            0x80 => {
                // R-block: b8=1, b7=0, b6 must be 1, b3 must be 0, b2 must be 1
                if (byte & 0x20) == 0 || (byte & 0x04) != 0 || (byte & 0x02) == 0 {
                    return Err(IsoDepErrorKind::ProtocolError);
                }
                Ok(Self::R(RBlock {
                    block_num: byte & 0x01,
                    ack: (byte & PCB_R_NAK_MASK) == 0,
                    has_cid: (byte & PCB_R_CID_MASK) != 0,
                }))
            }
            0xC0 => {
                // S-block: b8=1, b7=1, b3=0, b1=0
                // ISO/IEC 14443-4 Section 7.2: For WTX (b6=1, b5=1), b2 may be 1 or 0.
                // For DESELECT (b6=0, b5=0), b2 must be 1.
                if (byte & 0x05) != 0 {
                    return Err(IsoDepErrorKind::ProtocolError);
                }
                let kind = match (byte >> 4) & 0x03 {
                    0b00 => {
                        if (byte & 0x02) == 0 {
                            return Err(IsoDepErrorKind::ProtocolError);
                        }
                        SBlockKind::Deselect
                    }
                    0b11 => SBlockKind::Wtx,
                    _ => return Err(IsoDepErrorKind::ProtocolError),
                };
                Ok(Self::S(SBlock {
                    kind,
                    has_cid: (byte & PCB_S_CID_MASK) != 0,
                }))
            }
            _ => Err(IsoDepErrorKind::ProtocolError),
        }
    }

    /// Encodes this PCB into its 8-bit wire representation.
    #[inline]
    #[must_use]
    pub fn encode(&self) -> u8 {
        match self {
            Self::I(i) => i.encode(),
            Self::R(r) => r.encode(),
            Self::S(s) => s.encode(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i_block_encoding_and_parsing() {
        let block0 = IBlock {
            block_num: 0,
            chaining: false,
            has_cid: false,
            has_nad: false,
        };
        assert_eq!(block0.encode(), 0x02);
        assert_eq!(Pcb::parse(0x02).unwrap(), Pcb::I(block0));

        let block1_chained = IBlock {
            block_num: 1,
            chaining: true,
            has_cid: false,
            has_nad: false,
        };
        assert_eq!(block1_chained.encode(), 0x13);
        assert_eq!(Pcb::parse(0x13).unwrap(), Pcb::I(block1_chained));

        let block0_cid = IBlock {
            block_num: 0,
            chaining: false,
            has_cid: true,
            has_nad: false,
        };
        assert_eq!(block0_cid.encode(), 0x0A);
        assert_eq!(Pcb::parse(0x0A).unwrap(), Pcb::I(block0_cid));
    }

    #[test]
    fn test_r_block_encoding_and_parsing() {
        let ack0 = RBlock {
            block_num: 0,
            ack: true,
            has_cid: false,
        };
        assert_eq!(ack0.encode(), 0xA2);
        assert_eq!(Pcb::parse(0xA2).unwrap(), Pcb::R(ack0));

        let ack1 = RBlock {
            block_num: 1,
            ack: true,
            has_cid: false,
        };
        assert_eq!(ack1.encode(), 0xA3);
        assert_eq!(Pcb::parse(0xA3).unwrap(), Pcb::R(ack1));

        let nak0 = RBlock {
            block_num: 0,
            ack: false,
            has_cid: false,
        };
        assert_eq!(nak0.encode(), 0xB2);
        assert_eq!(Pcb::parse(0xB2).unwrap(), Pcb::R(nak0));
    }

    #[test]
    fn test_s_block_encoding_and_parsing() {
        let wtx = SBlock {
            kind: SBlockKind::Wtx,
            has_cid: false,
        };
        assert_eq!(wtx.encode(), 0xF2);
        assert_eq!(Pcb::parse(0xF2).unwrap(), Pcb::S(wtx));

        let deselect = SBlock {
            kind: SBlockKind::Deselect,
            has_cid: false,
        };
        assert_eq!(deselect.encode(), 0xC2);
        assert_eq!(Pcb::parse(0xC2).unwrap(), Pcb::S(deselect));
    }
}
