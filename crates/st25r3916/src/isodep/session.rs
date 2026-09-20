//! ISO/IEC 14443-4 Section 7.5: Half-duplex block protocol session state machine.
//!
//! Citations:
//! - ISO/IEC 14443-4:2008 / 2018:
//!   - Section 7.5: "Protocol operation"
//!   - Section 7.5.2: "Chaining"
//!   - Section 7.5.3: "Block numbering rules"
//!   - Section 7.5.4: "Block handling rules"
//!   - Section 7.3: "Frame waiting time extension" (WTX)
//!   - Section 8: "Protocol deactivation of PICC Type A and Type B" (DESELECT)

use embedded_hal::i2c::I2c;

use super::block::{IBlock, Pcb, RBlock, SBlock, SBlockKind};
use super::error::{IsoDepError, IsoDepErrorKind};
use super::rats::{build_rats, Ats, FSDI_256_BYTES};
use crate::St25r3916;

/// Maximum temporary buffer size for an ISO-DEP frame (prologue + INF).
pub const MAX_ISODEP_FRAME: usize = 260;

/// Default poll iterations for nominal Frame Waiting Time.
///
/// Each I2C poll iteration of `REG_MAIN_IRQ`/`FIFO_STATUS` takes ~50-150 µs over 400 kHz I2C.
/// 2,000 iterations provides a ~200-300 ms response window for initial ATS and supervisory frames.
pub const DEFAULT_FWT_POLL_ITERATIONS: u32 = 2_000;

/// Active ISO-DEP (ISO/IEC 14443-4) session with a PICC Type A card.
pub struct IsoDepSession {
    /// Maximum frame size accepted by PICC in bytes (from ATS).
    pub fsc: usize,
    /// Maximum frame size accepted by PCD in bytes (256).
    pub fsd: usize,
    /// Logical Card Identifier (CID) if assigned and supported.
    pub cid: Option<u8>,
    /// Frame Waiting time Integer from ATS.
    pub fwi: u8,
    /// PCD transmit block number (0 or 1).
    pub pcd_block_num: u8,
    /// PICC receive block number (0 or 1).
    pub picc_block_num: u8,
}

impl IsoDepSession {
    /// Activates ISO-DEP by sending RATS to a selected PICC Type A and parsing ATS.
    pub fn activate<I2C: I2c>(
        st: &mut St25r3916<I2C>,
        cid: Option<u8>,
    ) -> Result<(Ats, Self), IsoDepError<I2C::Error>> {
        let cid_val = cid.unwrap_or(0);
        let rats_cmd = build_rats(FSDI_256_BYTES, cid_val);

        // Transmit RATS with CRC
        st.transmit_frame_crc(&rats_cmd).map_err(IsoDepError::Bus)?;

        // Receive ATS response
        let mut ats_buf = [0u8; 32];
        let n = st
            .receive_frame_crc(&mut ats_buf, DEFAULT_FWT_POLL_ITERATIONS)
            .map_err(IsoDepError::Bus)?
            .ok_or(IsoDepErrorKind::RatsFailed)?;

        let ats = Ats::parse(&ats_buf[..n])?;

        let negotiated_cid = if ats.cid_supported() { cid } else { None };

        let session = Self {
            fsc: ats.fsc,
            fsd: 256,
            cid: negotiated_cid,
            fwi: ats.fwi(),
            pcd_block_num: 0,
            picc_block_num: 0,
        };

        Ok((ats, session))
    }

    /// Computes poll iterations based on nominal FWI and optional WTX multiplier.
    ///
    /// Each I2C poll iteration of `REG_MAIN_IRQ`/`FIFO_STATUS` takes ~50-150 µs.
    /// Formula: `FWT = 302 µs * 2^FWI`.
    /// For `FWI=8` (YubiKey default), `FWT` is ~77 ms.
    /// Provides at least 3,500 iterations (~350-500 ms) so cryptographic smart cards
    /// have adequate time for applet selection and internal flash lookups.
    #[inline]
    #[must_use]
    pub fn compute_poll_iterations(&self, wtxm: Option<u8>) -> u32 {
        let mult = wtxm.unwrap_or(1) as u32;
        let fwi = self.fwi.min(14) as u32;
        // Base iterations scaled by 2^fwi and WTXM
        let base = (200u32 << (fwi.saturating_sub(4))).clamp(3_500, 20_000);
        base.saturating_mul(mult).min(50_000)
    }

    /// Transmits an APDU command and receives the APDU response over ISO-DEP.
    ///
    /// Handles I-block transmission with chaining (if command exceeds `fsc`),
    /// card Waiting Time Extension (WTX) supervisory exchanges, and response
    /// chaining. Returns `(data_len, status_word)`.
    pub fn transceive_apdu<I2C: I2c>(
        &mut self,
        st: &mut St25r3916<I2C>,
        cmd_apdu: &[u8],
        resp_buf: &mut [u8],
    ) -> Result<(usize, u16), IsoDepError<I2C::Error>> {
        let prologue_len = 1 + if self.cid.is_some() { 1 } else { 0 };
        let max_inf = self.fsc.saturating_sub(prologue_len);
        if max_inf == 0 {
            return Err(IsoDepErrorKind::ProtocolError.into());
        }

        // 1. Transmit Command APDU in I-block(s)
        let mut cmd_offset = 0;
        while cmd_offset < cmd_apdu.len() {
            let remaining = cmd_apdu.len() - cmd_offset;
            let chunk_len = remaining.min(max_inf);
            let is_chained = remaining > chunk_len;

            let mut frame = [0u8; MAX_ISODEP_FRAME];
            let iblock = IBlock {
                block_num: self.pcd_block_num,
                chaining: is_chained,
                has_cid: self.cid.is_some(),
                has_nad: false,
            };
            frame[0] = iblock.encode();
            let mut flen = 1;
            if let Some(c) = self.cid {
                frame[flen] = c;
                flen += 1;
            }
            frame[flen..flen + chunk_len]
                .copy_from_slice(&cmd_apdu[cmd_offset..cmd_offset + chunk_len]);
            flen += chunk_len;

            st.transmit_frame_crc(&frame[..flen])
                .map_err(IsoDepError::Bus)?;

            cmd_offset += chunk_len;

            if is_chained {
                // Wait for R(ACK) from PICC before sending next chunk
                let mut ack_buf = [0u8; 16];
                let _ack_n = st
                    .receive_frame_crc(&mut ack_buf, DEFAULT_FWT_POLL_ITERATIONS)
                    .map_err(IsoDepError::Bus)?
                    .ok_or(IsoDepErrorKind::Timeout)?;
                let pcb = Pcb::parse(ack_buf[0])?;
                match pcb {
                    Pcb::R(r) => {
                        let expected_next_num = self.pcd_block_num ^ 1;
                        if !r.ack || r.block_num != expected_next_num {
                            return Err(IsoDepErrorKind::ChainingError.into());
                        }
                        self.pcd_block_num = expected_next_num;
                    }
                    _ => return Err(IsoDepErrorKind::ProtocolError.into()),
                }
            }
        }

        // 2. Receive Response APDU in I-block(s), handling WTX and response chaining
        let mut resp_offset = 0;
        let mut current_wtxm: Option<u8> = None;

        loop {
            let poll_iters = self.compute_poll_iterations(current_wtxm);
            current_wtxm = None;

            let mut rx_frame = [0u8; MAX_ISODEP_FRAME];
            let n = st
                .receive_frame_crc(&mut rx_frame, poll_iters)
                .map_err(IsoDepError::Bus)?
                .ok_or(IsoDepErrorKind::Timeout)?;

            let pcb = Pcb::parse(rx_frame[0])?;
            match pcb {
                Pcb::S(s) => match s.kind {
                    SBlockKind::Wtx => {
                        // PICC requests waiting time extension: INF contains WTXM
                        let wtxm_offset = 1 + if s.has_cid { 1 } else { 0 };
                        if n <= wtxm_offset {
                            return Err(IsoDepErrorKind::ProtocolError.into());
                        }
                        let wtxm = rx_frame[wtxm_offset] & 0x3F;
                        current_wtxm = Some(wtxm);

                        // Echo WTX response
                        let mut resp_s = [0u8; 4];
                        let sblock = SBlock {
                            kind: SBlockKind::Wtx,
                            has_cid: self.cid.is_some(),
                        };
                        resp_s[0] = sblock.encode();
                        let mut slen = 1;
                        if let Some(c) = self.cid {
                            resp_s[slen] = c;
                            slen += 1;
                        }
                        resp_s[slen] = wtxm;
                        slen += 1;

                        st.transmit_frame_crc(&resp_s[..slen])
                            .map_err(IsoDepError::Bus)?;
                        // Continue waiting for response
                    }
                    SBlockKind::Deselect => return Err(IsoDepErrorKind::Deselected.into()),
                },
                Pcb::I(i) => {
                    let inf_start =
                        1 + if i.has_cid { 1 } else { 0 } + if i.has_nad { 1 } else { 0 };
                    if n < inf_start {
                        return Err(IsoDepErrorKind::ProtocolError.into());
                    }
                    let inf = &rx_frame[inf_start..n];
                    if resp_offset + inf.len() > resp_buf.len() {
                        return Err(IsoDepErrorKind::BufferTooSmall.into());
                    }
                    resp_buf[resp_offset..resp_offset + inf.len()].copy_from_slice(inf);
                    resp_offset += inf.len();

                    self.picc_block_num ^= 1;

                    if i.chaining {
                        // Acknowledge chunk with R(ACK)
                        let mut ack = [0u8; 4];
                        let rblock = RBlock {
                            block_num: self.picc_block_num,
                            ack: true,
                            has_cid: self.cid.is_some(),
                        };
                        ack[0] = rblock.encode();
                        let mut alen = 1;
                        if let Some(c) = self.cid {
                            ack[alen] = c;
                            alen += 1;
                        }
                        st.transmit_frame_crc(&ack[..alen])
                            .map_err(IsoDepError::Bus)?;
                    } else {
                        // Completed transmission of all chained response blocks
                        self.pcd_block_num ^= 1;
                        if resp_offset < 2 {
                            return Err(IsoDepErrorKind::ProtocolError.into());
                        }
                        let data_len = resp_offset - 2;
                        let sw = u16::from_be_bytes([resp_buf[data_len], resp_buf[data_len + 1]]);
                        return Ok((data_len, sw));
                    }
                }
                Pcb::R(_) => return Err(IsoDepErrorKind::ProtocolError.into()),
            }
        }
    }

    /// Transmits an APDU command, automatically issuing ISO 7816-4 `GET RESPONSE` (`0x00 0xC0`)
    /// if the card returns status word `0x61XX`.
    pub fn transceive_apdu_auto_get_response<I2C: I2c>(
        &mut self,
        st: &mut St25r3916<I2C>,
        cmd_apdu: &[u8],
        resp_buf: &mut [u8],
    ) -> Result<(usize, u16), IsoDepError<I2C::Error>> {
        let (mut data_len, mut sw) = self.transceive_apdu(st, cmd_apdu, resp_buf)?;

        // If card indicates more bytes available (SW1 == 0x61), issue GET RESPONSE
        while (sw >> 8) == 0x61 {
            let le = (sw & 0xFF) as u8;
            let get_resp_cmd = [0x00, 0xC0, 0x00, 0x00, le];
            let mut temp_buf = [0u8; MAX_ISODEP_FRAME];
            let (chunk_len, next_sw) = self.transceive_apdu(st, &get_resp_cmd, &mut temp_buf)?;

            if data_len + chunk_len > resp_buf.len() {
                return Err(IsoDepErrorKind::BufferTooSmall.into());
            }
            resp_buf[data_len..data_len + chunk_len].copy_from_slice(&temp_buf[..chunk_len]);
            data_len += chunk_len;
            sw = next_sw;
        }

        Ok((data_len, sw))
    }

    /// Deselects the PICC via supervisory S(DESELECT) command.
    pub fn deselect<I2C: I2c>(
        &mut self,
        st: &mut St25r3916<I2C>,
    ) -> Result<(), IsoDepError<I2C::Error>> {
        let mut frame = [0u8; 4];
        let sblock = SBlock {
            kind: SBlockKind::Deselect,
            has_cid: self.cid.is_some(),
        };
        frame[0] = sblock.encode();
        let mut flen = 1;
        if let Some(c) = self.cid {
            frame[flen] = c;
            flen += 1;
        }

        st.transmit_frame_crc(&frame[..flen])
            .map_err(IsoDepError::Bus)?;

        let mut resp = [0u8; 16];
        let _ = st.receive_frame_crc(&mut resp, DEFAULT_FWT_POLL_ITERATIONS);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::*;
    use crate::registers::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    const ADDR: u8 = 0x50;

    #[test]
    fn test_isodep_activate_mock() {
        let txns = [
            // transmit_frame_crc(RATS [0xE0, 0x80])
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES2, 2 << 3]),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(ADDR, std::vec![MODE_FIFO_LOAD, 0xE0, 0x80]),
            Transaction::write(ADDR, std::vec![CMD_TRANSMIT_WITH_CRC]),
            // receive_frame_crc(ATS: TL=5, T0=0x78, TA1=0x80, TB1=0x80, TC1=0x02 + 2 bytes CRC)
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![7],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![MAIN_IRQ_RXE],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![7],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![MODE_FIFO_READ],
                std::vec![0x05, 0x78, 0x80, 0x80, 0x02, 0x12, 0x34],
            ),
        ];

        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDR);
        let (ats, session) = IsoDepSession::activate(&mut st, None).unwrap();
        assert_eq!(ats.tl, 5);
        assert_eq!(ats.fsc, 256);
        assert_eq!(session.fsc, 256);
        assert_eq!(session.pcd_block_num, 0);
        assert_eq!(session.picc_block_num, 0);
        st.release().done();
    }

    #[test]
    fn test_isodep_transceive_single_iblock_mock() {
        let ats = Ats {
            tl: 5,
            t0: 0x78,
            fsc: 256,
            ta1: Some(0x80),
            tb1: Some(0x80),
            tc1: Some(0x02),
            historical_bytes: heapless::Vec::new(),
        };
        let mut session = IsoDepSession {
            fsc: ats.fsc,
            fsd: 256,
            cid: None,
            fwi: ats.fwi(),
            pcd_block_num: 0,
            picc_block_num: 0,
        };

        let cmd = [0x00, 0xA4, 0x04, 0x00]; // SELECT command APDU
        let txns = [
            // transmit_frame_crc: PCB=0x02 + cmd (5 bytes)
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES2, 5 << 3]),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDR,
                std::vec![MODE_FIFO_LOAD, 0x02, 0x00, 0xA4, 0x04, 0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_TRANSMIT_WITH_CRC]),
            // receive_frame_crc: PCB=0x02 + data (0x01) + SW (0x90, 0x00) + CRC -> 6 bytes
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![6],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![MAIN_IRQ_RXE],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![6],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![MODE_FIFO_READ],
                std::vec![0x02, 0x01, 0x90, 0x00, 0x12, 0x34],
            ),
        ];

        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDR);
        let mut resp = [0u8; 32];
        let (len, sw) = session.transceive_apdu(&mut st, &cmd, &mut resp).unwrap();
        assert_eq!(len, 1);
        assert_eq!(resp[0], 0x01);
        assert_eq!(sw, 0x9000);
        assert_eq!(session.pcd_block_num, 1);
        assert_eq!(session.picc_block_num, 1);
        st.release().done();
    }

    #[test]
    fn test_isodep_transceive_with_wtx_mock() {
        let mut session = IsoDepSession {
            fsc: 256,
            fsd: 256,
            cid: None,
            fwi: 4,
            pcd_block_num: 0,
            picc_block_num: 0,
        };

        let cmd = [0x00, 0x84, 0x00, 0x00, 0x08]; // GET CHALLENGE
        let txns = [
            // transmit command I-block (6 bytes: PCB 0x02 + 5 bytes)
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES2, 6 << 3]),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDR,
                std::vec![MODE_FIFO_LOAD, 0x02, 0x00, 0x84, 0x00, 0x00, 0x08],
            ),
            Transaction::write(ADDR, std::vec![CMD_TRANSMIT_WITH_CRC]),
            // Card responds with S(WTX) request: PCB=0xF2, WTXM=0x04 (2 bytes + 2 bytes CRC -> 4 bytes)
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![4],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![MAIN_IRQ_RXE],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![4],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![MODE_FIFO_READ],
                std::vec![0xF2, 0x04, 0x12, 0x34],
            ),
            // Host echoes S(WTX) response: PCB=0xF2, WTXM=0x04 (2 bytes)
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES2, 2 << 3]),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(ADDR, std::vec![MODE_FIFO_LOAD, 0xF2, 0x04]),
            Transaction::write(ADDR, std::vec![CMD_TRANSMIT_WITH_CRC]),
            // Card finally responds with I-block: PCB=0x02 + SW (0x90, 0x00) + CRC -> 5 bytes
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![5],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![MAIN_IRQ_RXE],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![5],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![MODE_FIFO_READ],
                std::vec![0x02, 0x90, 0x00, 0x12, 0x34],
            ),
        ];

        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDR);
        let mut resp = [0u8; 32];
        let (len, sw) = session.transceive_apdu(&mut st, &cmd, &mut resp).unwrap();
        assert_eq!(len, 0);
        assert_eq!(sw, 0x9000);
        st.release().done();
    }

    #[test]
    fn test_isodep_transceive_chained_response_mock() {
        let mut session = IsoDepSession {
            fsc: 256,
            fsd: 256,
            cid: None,
            fwi: 4,
            pcd_block_num: 0,
            picc_block_num: 0,
        };

        let cmd = [0x00, 0xCA, 0x00, 0x6E, 0x00]; // GET DATA (OpenPGP)
        let txns = [
            // transmit command I-block (6 bytes: PCB 0x02 + 5 bytes)
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES2, 6 << 3]),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(
                ADDR,
                std::vec![MODE_FIFO_LOAD, 0x02, 0x00, 0xCA, 0x00, 0x6E, 0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_TRANSMIT_WITH_CRC]),
            // Card responds with chained I-block 0 (M=1): PCB=0x12, data=[0xAA, 0xBB] + CRC -> 5 bytes
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![5],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![MAIN_IRQ_RXE],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![5],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![MODE_FIFO_READ],
                std::vec![0x12, 0xAA, 0xBB, 0x12, 0x34],
            ),
            // Host acknowledges with R(ACK, 1): PCB=0xA3
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES1, 0x00]),
            Transaction::write(ADDR, std::vec![REG_NUM_TX_BYTES2, 1 << 3]),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![0x00],
            ),
            Transaction::write(ADDR, std::vec![CMD_CLEAR_FIFO]),
            Transaction::write(ADDR, std::vec![MODE_FIFO_LOAD, 0xA3]),
            Transaction::write(ADDR, std::vec![CMD_TRANSMIT_WITH_CRC]),
            // Card sends final I-block 1 (M=0): PCB=0x03, data=[0xCC] + SW [0x90, 0x00] + CRC -> 6 bytes
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![6],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_MAIN_IRQ)],
                std::vec![MAIN_IRQ_RXE],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS1)],
                std::vec![6],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![register_read_cmd(REG_FIFO_STATUS2)],
                std::vec![0],
            ),
            Transaction::write_read(
                ADDR,
                std::vec![MODE_FIFO_READ],
                std::vec![0x03, 0xCC, 0x90, 0x00, 0x12, 0x34],
            ),
        ];

        let i2c = Mock::new(&txns);
        let mut st = St25r3916::new(i2c, ADDR);
        let mut resp = [0u8; 32];
        let (len, sw) = session.transceive_apdu(&mut st, &cmd, &mut resp).unwrap();
        assert_eq!(len, 3);
        assert_eq!(&resp[..3], &[0xAA, 0xBB, 0xCC]);
        assert_eq!(sw, 0x9000);
        assert_eq!(session.pcd_block_num, 1);
        assert_eq!(session.picc_block_num, 0);
        st.release().done();
    }
}
