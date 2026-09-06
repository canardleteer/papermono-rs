# st25r3916

Register definitions, direct commands, Initiator (Reader) mode,
and Target (Card Emulation) profiles for the STMicroelectronics
ST25R3916 NFC transceiver.

Datasheet: STMicroelectronics ST25R3916 / ST25R3916B / ST25R3917
/ ST25R3917B Datasheet DS12484 Rev 8.

> [!CAUTION]
> While ISO 14443-A Initiator mode is hardware-verified on
> M5Stack PaperMono (`C153`), the target and card emulation profiles
> (`NfcATargetConfig`, `NfcFTargetConfig`, `Nfcip1TargetConfig`) and
> higher-layer protocol framing (Type 2, Type 4A, APDU, and NDEF) in
> this crate are verified via mock unit tests and have not yet been
> validated against physical external NFC reader devices.

This crate provides a `#![no_std]` driver based on `embedded-hal` 1.0.

## Supported Modes

1. **Initiator (Reader / Poller)**:
   - ISO/IEC 14443-A WUPA/REQA polling, anticollision loop, and SAK selection.
   - Single-size (4-byte) and double-size (7-byte) UID acquisition.
2. **Target (Card Emulation / Listener)**:
   - 48-byte `PT_Memory` (passive target memory) configuration and chunked
     loading.
   - ISO 14443-A passive target mode (Type 2 Tag 4-byte UID and Type 4A
     Tag 7-byte UID ISO-DEP).
   - FeliCa passive target mode (NFC-F at 212 kbps and 424 kbps).
   - NFCIP-1 passive and active communication target modes (ISO/IEC 18092 P2P).
3. **Protocol Framing**:
   - NFC Type 2 Tag block layout, capability container (CC), and NDEF TLV.
   - ISO/IEC 7816-4 APDU parsing and status responses for Type 4A tags.
   - Lightweight NDEF message and record construction.
