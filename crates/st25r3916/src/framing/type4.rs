//! ISO/IEC 7816-4 APDU decoding and NFC Forum Type 4 Tag application state machine.
//!
//! Citations:
//! - ISO/IEC 7816-4: "Identification cards — Integrated circuit cards — Part 4: Organization, security and commands for interchange"
//! - NFC Forum Type 4 Tag Technical Specification (T4T 2.0)

/// Type 4 Tag Application Identifier (AID) for NFC Forum NDEF application (`D2760000850101h`).
pub const AID_NDEF_V2: [u8; 7] = [0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01];

/// Capability Container (CC) File ID (`0xE103h`).
pub const FILE_ID_CC: [u8; 2] = [0xE1, 0x03];

/// NDEF File ID (`0xE104h`).
pub const FILE_ID_NDEF: [u8; 2] = [0xE1, 0x04];

/// ISO 7816-4 Instruction: Select File / Application (`0xA4`).
pub const INS_SELECT: u8 = 0xA4;

/// ISO 7816-4 Instruction: Read Binary (`0xB0`).
pub const INS_READ_BINARY: u8 = 0xB0;

/// ISO 7816-4 Instruction: Update Binary (`0xD6`).
pub const INS_UPDATE_BINARY: u8 = 0xD6;

/// Status Word: Success (`0x9000`).
pub const SW_SUCCESS: (u8, u8) = (0x90, 0x00);

/// Status Word: File Not Found (`0x6A82`).
pub const SW_FILE_NOT_FOUND: (u8, u8) = (0x6A, 0x82);

/// Status Word: Instruction Not Supported (`0x6D00`).
pub const SW_INS_NOT_SUPPORTED: (u8, u8) = (0x6D, 0x00);

/// Status Word: Wrong Length (`0x6700`).
pub const SW_WRONG_LENGTH: (u8, u8) = (0x67, 0x00);

/// Status Word: Wrong Parameters P1-P2 (`0x6B00`).
pub const SW_WRONG_PARAMS: (u8, u8) = (0x6B, 0x00);

/// APDU parsing errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApduError {
    /// APDU was too short (< 4 bytes for header).
    HeaderTooShort,
    /// Response buffer has insufficient capacity for payload and status word.
    BufferTooSmall,
}

/// Parsed ISO/IEC 7816-4 Command APDU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandApdu<'a> {
    /// Class byte (CLA).
    pub cla: u8,
    /// Instruction byte (INS).
    pub ins: u8,
    /// Parameter 1 (P1).
    pub p1: u8,
    /// Parameter 2 (P2).
    pub p2: u8,
    /// Command data payload.
    pub data: &'a [u8],
    /// Expected response length (Le), if present.
    pub le: Option<usize>,
}

impl<'a> CommandApdu<'a> {
    /// Parses a raw byte slice into a [`CommandApdu`].
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ApduError> {
        if bytes.len() < 4 {
            return Err(ApduError::HeaderTooShort);
        }
        let cla = bytes[0];
        let ins = bytes[1];
        let p1 = bytes[2];
        let p2 = bytes[3];

        if bytes.len() == 4 {
            // Case 1: Header only
            return Ok(Self {
                cla,
                ins,
                p1,
                p2,
                data: &[],
                le: None,
            });
        }

        if bytes.len() == 5 {
            // Case 2: Header + Le
            let le = if bytes[4] == 0 { 256 } else { bytes[4] as usize };
            return Ok(Self {
                cla,
                ins,
                p1,
                p2,
                data: &[],
                le: Some(le),
            });
        }

        let lc = bytes[4] as usize;
        if bytes.len() < 5 + lc {
            return Err(ApduError::HeaderTooShort);
        }

        let data = &bytes[5..5 + lc];
        let le = if bytes.len() > 5 + lc {
            let raw_le = bytes[5 + lc];
            Some(if raw_le == 0 { 256 } else { raw_le as usize })
        } else {
            None
        };

        Ok(Self {
            cla,
            ins,
            p1,
            p2,
            data,
            le,
        })
    }
}

/// Currently selected file in the Type 4 Tag state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedFile {
    /// No file selected (at root application level).
    None,
    /// Capability Container file selected (`0xE103h`).
    CapabilityContainer,
    /// NDEF file selected (`0xE104h`).
    Ndef,
}

/// NFC Forum Type 4 Tag Application state machine (default 256 bytes NDEF capacity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type4TagApp<const MAX_NDEF: usize = 256> {
    selected: SelectedFile,
    app_selected: bool,
    ndef_data: [u8; MAX_NDEF],
    ndef_len: usize,
}

impl<const MAX_NDEF: usize> Default for Type4TagApp<MAX_NDEF> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_NDEF: usize> Type4TagApp<MAX_NDEF> {
    /// Creates a new Type 4 Tag application instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: SelectedFile::None,
            app_selected: false,
            ndef_data: [0u8; MAX_NDEF],
            ndef_len: 0,
        }
    }

    /// Sets the active NDEF message content.
    ///
    /// The message is stored with a 2-byte big-endian NLEN prefix.
    pub fn set_ndef_message(&mut self, msg: &[u8]) -> bool {
        if msg.len() + 2 > MAX_NDEF {
            return false;
        }
        self.ndef_data[0] = (msg.len() >> 8) as u8;
        self.ndef_data[1] = msg.len() as u8;
        self.ndef_data[2..2 + msg.len()].copy_from_slice(msg);
        self.ndef_len = 2 + msg.len();
        true
    }

    /// Generates the 15-byte Capability Container (CC) file content per T4T v2.0.
    #[must_use]
    pub fn cc_file(&self) -> [u8; 15] {
        let max_size = (MAX_NDEF as u16).to_be_bytes();
        [
            0x00, 0x0F, // CCLEN: 15 bytes
            0x20, // Mapping version 2.0
            0x00, 0x7F, // MLe: 127 bytes max read
            0x00, 0x7F, // MLc: 127 bytes max write
            // NDEF File Control TLV:
            0x04, // T: NDEF File Control TLV
            0x06, // L: 6 bytes
            FILE_ID_NDEF[0], FILE_ID_NDEF[1], // File ID: 0xE104
            max_size[0], max_size[1], // Max NDEF size
            0x00, // Read access: free
            0x00, // Write access: free
        ]
    }

    /// Processes an incoming ISO 7816-4 APDU frame and writes the response into `resp`.
    ///
    /// Returns the number of bytes written to `resp` including status words (SW1, SW2).
    pub fn handle_apdu(&mut self, apdu_bytes: &[u8], resp: &mut [u8]) -> Result<usize, ApduError> {
        let apdu = CommandApdu::parse(apdu_bytes)?;
        if resp.len() < 2 {
            return Err(ApduError::BufferTooSmall);
        }

        match apdu.ins {
            INS_SELECT => self.handle_select(&apdu, resp),
            INS_READ_BINARY => self.handle_read_binary(&apdu, resp),
            _ => {
                resp[0] = SW_INS_NOT_SUPPORTED.0;
                resp[1] = SW_INS_NOT_SUPPORTED.1;
                Ok(2)
            }
        }
    }

    fn handle_select(
        &mut self,
        apdu: &CommandApdu<'_>,
        resp: &mut [u8],
    ) -> Result<usize, ApduError> {
        if apdu.p1 == 0x04 {
            // Select by AID
            if apdu.data == AID_NDEF_V2 {
                self.app_selected = true;
                self.selected = SelectedFile::None;
                resp[0] = SW_SUCCESS.0;
                resp[1] = SW_SUCCESS.1;
                return Ok(2);
            }
            resp[0] = SW_FILE_NOT_FOUND.0;
            resp[1] = SW_FILE_NOT_FOUND.1;
            return Ok(2);
        }

        if apdu.p1 == 0x00 {
            // Select by File ID
            if apdu.data == FILE_ID_CC {
                self.selected = SelectedFile::CapabilityContainer;
                resp[0] = SW_SUCCESS.0;
                resp[1] = SW_SUCCESS.1;
                return Ok(2);
            }
            if apdu.data == FILE_ID_NDEF {
                self.selected = SelectedFile::Ndef;
                resp[0] = SW_SUCCESS.0;
                resp[1] = SW_SUCCESS.1;
                return Ok(2);
            }
        }

        resp[0] = SW_FILE_NOT_FOUND.0;
        resp[1] = SW_FILE_NOT_FOUND.1;
        Ok(2)
    }

    fn handle_read_binary(
        &self,
        apdu: &CommandApdu<'_>,
        resp: &mut [u8],
    ) -> Result<usize, ApduError> {
        let offset = ((apdu.p1 as usize) << 8) | (apdu.p2 as usize);
        let le = apdu.le.unwrap_or(0);

        let data_source: &[u8] = match self.selected {
            SelectedFile::CapabilityContainer => &self.cc_file(),
            SelectedFile::Ndef => &self.ndef_data[..self.ndef_len],
            SelectedFile::None => {
                resp[0] = SW_FILE_NOT_FOUND.0;
                resp[1] = SW_FILE_NOT_FOUND.1;
                return Ok(2);
            }
        };

        if offset >= data_source.len() {
            resp[0] = SW_WRONG_PARAMS.0;
            resp[1] = SW_WRONG_PARAMS.1;
            return Ok(2);
        }

        let available = data_source.len() - offset;
        let count = if le == 0 || le > available {
            available
        } else {
            le
        };

        if resp.len() < count + 2 {
            return Err(ApduError::BufferTooSmall);
        }

        resp[..count].copy_from_slice(&data_source[offset..offset + count]);
        resp[count] = SW_SUCCESS.0;
        resp[count + 1] = SW_SUCCESS.1;
        Ok(count + 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_apdu() {
        let raw = [0x00, 0xA4, 0x04, 0x00, 0x07, 0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01];
        let apdu = CommandApdu::parse(&raw).unwrap();
        assert_eq!(apdu.cla, 0x00);
        assert_eq!(apdu.ins, INS_SELECT);
        assert_eq!(apdu.p1, 0x04);
        assert_eq!(apdu.p2, 0x00);
        assert_eq!(apdu.data, AID_NDEF_V2);
        assert_eq!(apdu.le, None);
    }

    #[test]
    fn test_type4_select_and_read_cc() {
        let mut app = Type4TagApp::<128>::new();
        let mut resp = [0u8; 64];

        // 1. Select AID
        let select_aid = [0x00, 0xA4, 0x04, 0x00, 0x07, 0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01];
        let n = app.handle_apdu(&select_aid, &mut resp).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&resp[..2], &[0x90, 0x00]);

        // 2. Select CC file
        let select_cc = [0x00, 0xA4, 0x00, 0x0C, 0x02, 0xE1, 0x03];
        let n = app.handle_apdu(&select_cc, &mut resp).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&resp[..2], &[0x90, 0x00]);

        // 3. Read CC binary (15 bytes)
        let read_cc = [0x00, 0xB0, 0x00, 0x00, 0x0F];
        let n = app.handle_apdu(&read_cc, &mut resp).unwrap();
        assert_eq!(n, 17);
        assert_eq!(resp[0], 0x00);
        assert_eq!(resp[1], 0x0F); // CCLEN = 15
        assert_eq!(resp[2], 0x20); // Version 2.0
        assert_eq!(&resp[15..17], &[0x90, 0x00]);
    }

    #[test]
    fn test_type4_select_and_read_ndef() {
        let mut app = Type4TagApp::<128>::new();
        app.set_ndef_message(b"Hello NDEF");

        let mut resp = [0u8; 64];
        // Select NDEF file
        let select_ndef = [0x00, 0xA4, 0x00, 0x0C, 0x02, 0xE1, 0x04];
        let n = app.handle_apdu(&select_ndef, &mut resp).unwrap();
        assert_eq!(&resp[..n], &[0x90, 0x00]);

        // Read binary length prefix
        let read_len = [0x00, 0xB0, 0x00, 0x00, 0x02];
        let n = app.handle_apdu(&read_len, &mut resp).unwrap();
        assert_eq!(n, 4);
        assert_eq!(resp[0], 0x00);
        assert_eq!(resp[1], 10); // length of "Hello NDEF"
        assert_eq!(&resp[2..4], &[0x90, 0x00]);
    }
}
