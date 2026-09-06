//! NFC protocol data structures, APDU processing, and NDEF record framing.

pub mod ndef;
pub mod type2;
pub mod type4;

pub use ndef::{
    build_text_record, build_uri_record, wrap_in_type2_tlv, NdefError, UriPrefix, NDEF_FLAG_MB,
    NDEF_FLAG_ME, NDEF_FLAG_SR, NDEF_TNF_WELL_KNOWN, RTD_TEXT, RTD_URI,
};
pub use type2::{Type2Error, Type2Memory, CMD_TYPE2_READ, CMD_TYPE2_WRITE, TYPE2_ACK};
pub use type4::{
    ApduError, CommandApdu, SelectedFile, Type4TagApp, AID_NDEF_V2, FILE_ID_CC, FILE_ID_NDEF,
    INS_READ_BINARY, INS_SELECT, INS_UPDATE_BINARY, SW_FILE_NOT_FOUND, SW_INS_NOT_SUPPORTED,
    SW_SUCCESS, SW_WRONG_LENGTH, SW_WRONG_PARAMS,
};
