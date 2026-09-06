//! Lightweight `no_std` NDEF (NFC Data Exchange Format) builders.
//!
//! Citations:
//! - NFC Forum NDEF Technical Specification (NDEF 1.0)
//! - NFC Forum URI Record Type Definition (RTD-URI 1.0)
//! - NFC Forum Text Record Type Definition (RTD-Text 1.0)

/// Error conditions during NDEF record construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NdefError {
    /// Provided destination buffer is too small for the record.
    BufferTooSmall,
    /// Language code exceeds maximum allowed 63 ASCII characters.
    LanguageCodeTooLong,
    /// Payload exceeds maximum single-record limit for Short Record format (255 bytes).
    PayloadTooLarge,
}

/// NFC Forum URI identifier code prefixes (RTD-URI Section 3.2.2 Table 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UriPrefix {
    /// No prepending prefix.
    None = 0x00,
    /// `http://www.`
    HttpWww = 0x01,
    /// `https://www.`
    HttpsWww = 0x02,
    /// `http://`
    Http = 0x03,
    /// `https://`
    Https = 0x04,
    /// `tel:`
    Tel = 0x05,
    /// `mailto:`
    Mailto = 0x06,
}

/// NDEF record header flag: Message Begin (MB).
pub const NDEF_FLAG_MB: u8 = 0x80;

/// NDEF record header flag: Message End (ME).
pub const NDEF_FLAG_ME: u8 = 0x40;

/// NDEF record header flag: Short Record (SR, 1-byte payload length).
pub const NDEF_FLAG_SR: u8 = 0x10;

/// NDEF record header flag: NFC Forum Well Known Type (TNF = 0x01).
pub const NDEF_TNF_WELL_KNOWN: u8 = 0x01;

/// Record Type Definition for URI (`b"U"`).
pub const RTD_URI: u8 = b'U';

/// Record Type Definition for Text (`b"T"`).
pub const RTD_TEXT: u8 = b'T';

/// Builds a standalone Short Record (SR) NDEF URI record into the provided buffer.
///
/// Returns the number of bytes written to `buf`.
pub fn build_uri_record(
    prefix: UriPrefix,
    uri_suffix: &str,
    buf: &mut [u8],
) -> Result<usize, NdefError> {
    let suffix_bytes = uri_suffix.as_bytes();
    let payload_len = 1 + suffix_bytes.len(); // 1 byte prefix code + suffix
    if payload_len > 255 {
        return Err(NdefError::PayloadTooLarge);
    }
    let total_len = 4 + payload_len; // Header(1) + TypeLen(1) + PayloadLen(1) + Type(1) + Payload
    if buf.len() < total_len {
        return Err(NdefError::BufferTooSmall);
    }

    buf[0] = NDEF_FLAG_MB | NDEF_FLAG_ME | NDEF_FLAG_SR | NDEF_TNF_WELL_KNOWN;
    buf[1] = 1; // Type length = 1
    buf[2] = payload_len as u8;
    buf[3] = RTD_URI;
    buf[4] = prefix as u8;
    buf[5..total_len].copy_from_slice(suffix_bytes);

    Ok(total_len)
}

/// Builds a standalone Short Record (SR) NDEF Text record into the provided buffer.
///
/// Returns the number of bytes written to `buf`.
pub fn build_text_record(lang: &str, text: &str, buf: &mut [u8]) -> Result<usize, NdefError> {
    let lang_bytes = lang.as_bytes();
    if lang_bytes.len() > 0x3F {
        return Err(NdefError::LanguageCodeTooLong);
    }
    let text_bytes = text.as_bytes();
    let payload_len = 1 + lang_bytes.len() + text_bytes.len(); // Status(1) + Lang + Text
    if payload_len > 255 {
        return Err(NdefError::PayloadTooLarge);
    }
    let total_len = 4 + payload_len; // Header(1) + TypeLen(1) + PayloadLen(1) + Type(1) + Payload
    if buf.len() < total_len {
        return Err(NdefError::BufferTooSmall);
    }

    buf[0] = NDEF_FLAG_MB | NDEF_FLAG_ME | NDEF_FLAG_SR | NDEF_TNF_WELL_KNOWN;
    buf[1] = 1; // Type length = 1
    buf[2] = payload_len as u8;
    buf[3] = RTD_TEXT;
    buf[4] = lang_bytes.len() as u8; // Status byte: UTF-8 (bit 7 = 0) and language length
    let lang_end = 5 + lang_bytes.len();
    buf[5..lang_end].copy_from_slice(lang_bytes);
    buf[lang_end..total_len].copy_from_slice(text_bytes);

    Ok(total_len)
}

/// Wraps an NDEF message in a Type 2 Tag NDEF Message TLV (`0x03`) followed by a Terminator TLV (`0xFE`).
///
/// Returns the total number of bytes written to `buf`.
pub fn wrap_in_type2_tlv(ndef_msg: &[u8], buf: &mut [u8]) -> Result<usize, NdefError> {
    if ndef_msg.len() > 254 {
        return Err(NdefError::PayloadTooLarge);
    }
    let total_len = 2 + ndef_msg.len() + 1; // Tag(1) + Len(1) + Payload + Terminator(1)
    if buf.len() < total_len {
        return Err(NdefError::BufferTooSmall);
    }

    buf[0] = 0x03; // NDEF Message TLV tag
    buf[1] = ndef_msg.len() as u8;
    buf[2..2 + ndef_msg.len()].copy_from_slice(ndef_msg);
    buf[2 + ndef_msg.len()] = 0xFE; // Terminator TLV

    Ok(total_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_record_construction() {
        let mut buf = [0u8; 32];
        let n = build_uri_record(UriPrefix::Https, "m5stack.com", &mut buf).unwrap();
        assert_eq!(n, 16);
        assert_eq!(buf[0], 0xD1); // MB | ME | SR | TNF_WELL_KNOWN
        assert_eq!(buf[1], 1); // Type length
        assert_eq!(buf[2], 12); // Payload length: 1 (prefix) + 11 ("m5stack.com")
        assert_eq!(buf[3], b'U');
        assert_eq!(buf[4], 0x04); // https://
        assert_eq!(&buf[5..16], b"m5stack.com");
    }

    #[test]
    fn test_text_record_construction() {
        let mut buf = [0u8; 32];
        let n = build_text_record("en", "PaperMono", &mut buf).unwrap();
        assert_eq!(n, 16);
        assert_eq!(buf[0], 0xD1);
        assert_eq!(buf[1], 1);
        assert_eq!(buf[2], 12); // Status(1) + "en"(2) + "PaperMono"(9)
        assert_eq!(buf[3], b'T');
        assert_eq!(buf[4], 2); // lang len = 2
        assert_eq!(&buf[5..7], b"en");
        assert_eq!(&buf[7..16], b"PaperMono");
    }

    #[test]
    fn test_wrap_in_type2_tlv() {
        let mut ndef = [0u8; 16];
        let n = build_uri_record(UriPrefix::Https, "test", &mut ndef).unwrap();

        let mut tlv_buf = [0u8; 32];
        let total = wrap_in_type2_tlv(&ndef[..n], &mut tlv_buf).unwrap();
        assert_eq!(total, n + 3);
        assert_eq!(tlv_buf[0], 0x03);
        assert_eq!(tlv_buf[1], n as u8);
        assert_eq!(&tlv_buf[2..2 + n], &ndef[..n]);
        assert_eq!(tlv_buf[2 + n], 0xFE);
    }
}
