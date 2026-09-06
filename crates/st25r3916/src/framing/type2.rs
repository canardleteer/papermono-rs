//! NFC Forum Type 2 Tag memory model, command decoder, and framing.
//!
//! Citations:
//! - NFC Forum Type 2 Tag Operation Specification
//! - NXP NTAG213/215/216 Datasheet

/// Standard 4-byte block size for Type 2 Tag memory.
pub const TYPE2_PAGE_SIZE: usize = 4;

/// Type 2 Tag Read command opcode (`0x30`). Reads 4 consecutive blocks (16 bytes).
pub const CMD_TYPE2_READ: u8 = 0x30;

/// Type 2 Tag Write command opcode (`0xA2`). Writes 1 block (4 bytes).
pub const CMD_TYPE2_WRITE: u8 = 0xA2;

/// Type 2 Tag 4-bit ACK response code (`0x0A`).
pub const TYPE2_ACK: u8 = 0x0A;

/// Errors produced during Type 2 Tag command processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type2Error {
    /// Command frame was incomplete or truncated.
    InvalidCommandLength,
    /// Unknown or unsupported command opcode.
    UnsupportedCommand(u8),
    /// Requested block number is out of bounds.
    BlockOutOfBounds(u8),
    /// Attempt to write to a read-only or locked block.
    BlockReadOnly(u8),
    /// Destination response buffer is too small.
    BufferTooSmall,
}

/// NFC Forum Type 2 Tag static memory model (default 16 pages = 64 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type2Memory<const PAGES: usize = 16> {
    data: [[u8; TYPE2_PAGE_SIZE]; PAGES],
}

impl<const PAGES: usize> Default for Type2Memory<PAGES> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const PAGES: usize> Type2Memory<PAGES> {
    /// Creates a new Type 2 Tag memory initialized with standard CC container in Block 3.
    #[must_use]
    pub const fn new() -> Self {
        let mut mem = Self {
            data: [[0u8; TYPE2_PAGE_SIZE]; PAGES],
        };
        if PAGES > 3 {
            // Block 3: Capability Container (CC)
            // Byte 0: Magic byte 0xE1
            // Byte 1: Version 1.0 (0x10)
            // Byte 2: Data size = (PAGES - 4) * 4 / 8
            // Byte 3: Read/Write access = 0x00
            let data_size = if PAGES >= 4 {
                ((PAGES - 4) * TYPE2_PAGE_SIZE / 8) as u8
            } else {
                0
            };
            mem.data[3] = [0xE1, 0x10, data_size, 0x00];
        }
        mem
    }

    /// Initializes Block 0 and Block 1 with a 4-byte UID and calculated BCC0.
    pub fn set_uid_single(&mut self, uid: [u8; 4]) {
        if PAGES >= 2 {
            let bcc0 = 0x88 ^ uid[0] ^ uid[1] ^ uid[2];
            self.data[0] = [uid[0], uid[1], uid[2], bcc0];
            self.data[1] = [uid[3], 0x00, 0x00, 0x00];
        }
    }

    /// Copies NDEF TLV payload starting at Block 4 (user data area).
    pub fn set_ndef_tlv(&mut self, tlv_data: &[u8]) {
        let mut src_idx = 0;
        let mut page = 4;
        while page < PAGES && src_idx < tlv_data.len() {
            let mut byte_idx = 0;
            while byte_idx < TYPE2_PAGE_SIZE && src_idx < tlv_data.len() {
                self.data[page][byte_idx] = tlv_data[src_idx];
                src_idx += 1;
                byte_idx += 1;
            }
            while byte_idx < TYPE2_PAGE_SIZE {
                self.data[page][byte_idx] = 0x00;
                byte_idx += 1;
            }
            page += 1;
        }
    }

    /// Reads a single 4-byte page.
    #[inline]
    #[must_use]
    pub fn read_page(&self, page: usize) -> Option<&[u8; TYPE2_PAGE_SIZE]> {
        self.data.get(page)
    }

    /// Writes a single 4-byte page. Returns `false` if `page` is out of bounds or read-only (page 0..3).
    pub fn write_page(&mut self, page: usize, val: [u8; TYPE2_PAGE_SIZE]) -> bool {
        if page < 4 || page >= PAGES {
            return false;
        }
        self.data[page] = val;
        true
    }

    /// Handles an incoming NFC Forum Type 2 Tag command.
    ///
    /// Implements `CMD_TYPE2_READ` (`0x30`, returns 16 bytes: 4 blocks) and
    /// `CMD_TYPE2_WRITE` (`0xA2`, writes 4 bytes, returns 1-byte ACK `0x0A`).
    pub fn handle_command(&mut self, cmd: &[u8], resp: &mut [u8]) -> Result<usize, Type2Error> {
        if cmd.is_empty() {
            return Err(Type2Error::InvalidCommandLength);
        }

        match cmd[0] {
            CMD_TYPE2_READ => {
                if cmd.len() < 2 {
                    return Err(Type2Error::InvalidCommandLength);
                }
                let start_page = cmd[1] as usize;
                if start_page >= PAGES {
                    return Err(Type2Error::BlockOutOfBounds(cmd[1]));
                }
                if resp.len() < 16 {
                    return Err(Type2Error::BufferTooSmall);
                }

                // Read 4 consecutive pages (16 bytes), wrapping within memory size.
                for i in 0..4 {
                    let page_idx = (start_page + i) % PAGES;
                    let out_start = i * TYPE2_PAGE_SIZE;
                    resp[out_start..out_start + TYPE2_PAGE_SIZE].copy_from_slice(&self.data[page_idx]);
                }
                Ok(16)
            }
            CMD_TYPE2_WRITE => {
                if cmd.len() < 6 {
                    return Err(Type2Error::InvalidCommandLength);
                }
                let page = cmd[1] as usize;
                if page >= PAGES {
                    return Err(Type2Error::BlockOutOfBounds(cmd[1]));
                }
                if page < 4 {
                    return Err(Type2Error::BlockReadOnly(cmd[1]));
                }
                if resp.is_empty() {
                    return Err(Type2Error::BufferTooSmall);
                }

                let mut val = [0u8; TYPE2_PAGE_SIZE];
                val.copy_from_slice(&cmd[2..6]);
                self.data[page] = val;

                resp[0] = TYPE2_ACK;
                Ok(1)
            }
            other => Err(Type2Error::UnsupportedCommand(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type2_cc_initialization() {
        let mem = Type2Memory::<16>::new();
        let cc = mem.read_page(3).unwrap();
        assert_eq!(cc[0], 0xE1);
        assert_eq!(cc[1], 0x10);
        assert_eq!(cc[2], 6); // (16 - 4) * 4 / 8 = 6 (48 bytes)
        assert_eq!(cc[3], 0x00);
    }

    #[test]
    fn test_type2_read_command() {
        let mut mem = Type2Memory::<16>::new();
        mem.set_uid_single([0x01, 0x02, 0x03, 0x04]);

        let cmd = [CMD_TYPE2_READ, 0x00];
        let mut resp = [0u8; 16];
        let n = mem.handle_command(&cmd, &mut resp).unwrap();
        assert_eq!(n, 16);
        assert_eq!(resp[0], 0x01);
        assert_eq!(resp[1], 0x02);
        assert_eq!(resp[2], 0x03);
        assert_eq!(resp[3], 0x88 ^ 0x01 ^ 0x02 ^ 0x03);
        assert_eq!(resp[4], 0x04);
    }

    #[test]
    fn test_type2_write_command() {
        let mut mem = Type2Memory::<16>::new();
        let write_cmd = [CMD_TYPE2_WRITE, 0x04, 0xAA, 0xBB, 0xCC, 0xDD];
        let mut resp = [0u8; 4];
        let n = mem.handle_command(&write_cmd, &mut resp).unwrap();
        assert_eq!(n, 1);
        assert_eq!(resp[0], TYPE2_ACK);

        assert_eq!(mem.read_page(4).unwrap(), &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_type2_write_readonly_block_rejected() {
        let mut mem = Type2Memory::<16>::new();
        let write_cmd = [CMD_TYPE2_WRITE, 0x03, 0xAA, 0xBB, 0xCC, 0xDD];
        let mut resp = [0u8; 4];
        assert_eq!(
            mem.handle_command(&write_cmd, &mut resp),
            Err(Type2Error::BlockReadOnly(0x03))
        );
    }
}
