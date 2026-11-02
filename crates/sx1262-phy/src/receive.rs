//! Managed IRQ snapshots, FIFO reads and timed reception cleanup.
use crate::*;

/// Outcome from one receive IRQ snapshot. Rejected frames never access FIFO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceivePoll {
    /// No terminal event; observed nonterminal bits were acknowledged.
    Pending,
    /// CRC/header failed. Bits describe the snapshot that was acknowledged.
    Rejected {
        /// Observed interrupt bits.
        irq: u16,
    },
    /// Hardware RX timeout; timed-RX RTC cleanup completed.
    Timeout {
        /// Observed interrupt bits.
        irq: u16,
    },
    /// Valid packet, possibly copied only as a bounded preview.
    Packet {
        /// Chip-reported packet length, including configured implicit length.
        len: u8,
        /// Bytes actually copied into the caller's buffer.
        copied: usize,
        /// Lossless packet metrics.
        status: PacketStatus,
    },
}
impl<S, B, D, H> Sx1262<S, B, D, H>
where
    S: embedded_hal_async::spi::SpiDevice,
    B: embedded_hal::digital::InputPin,
    D: embedded_hal_async::delay::DelayNs,
{
    /// Stops reception without changing antenna/power policy. After timed RX,
    /// stops RTC and clears its event while preserving unrelated register bits.
    /// Catalog `sx1262` §15.3.2 “Workaround” applies after any timed RX sequence,
    /// including early stop. On failure the flag remains set and readiness fails.
    pub async fn stop_rx(&mut self) -> Result<(), SxError<S::Error, B::Error>> {
        self.write_unchecked(CMD_SET_STANDBY, &[STDBY_CONFIG_RC])
            .await?;
        if self.timed_rx {
            self.write_reg(REG_RTC_CONTROL, 0).await?;
            let value = self.read_reg(REG_RTC_EVENT_CLEAR).await?;
            self.write_reg(REG_RTC_EVENT_CLEAR, value | RTC_EVENT_CLEAR_MASK)
                .await?;
            self.timed_rx = false;
        }
        Ok(())
    }
    /// Polls IRQs once, rejects CRC/header failures before FIFO access, and copies
    /// at most the caller's buffer size. FIFO addresses wrap at 256. Acknowledges
    /// only observed IRQ bits, including during continuous RX. Repeated arrivals
    /// of the same sticky bit cannot be distinguished by this snapshot protocol.
    /// Await to completion; no select/timeout cancellation around this sequence.
    pub async fn poll_receive(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<ReceivePoll, SxError<S::Error, B::Error>> {
        let irq = self.get_irq_status().await?;
        if irq == 0 {
            return Ok(ReceivePoll::Pending);
        }
        let terminal = irq & (IRQ_RX_DONE | IRQ_TIMEOUT | IRQ_CRC_ERR | IRQ_HEADER_ERR) != 0;
        let result = if irq & (IRQ_CRC_ERR | IRQ_HEADER_ERR) != 0 {
            ReceivePoll::Rejected { irq }
        } else if irq & IRQ_TIMEOUT != 0 {
            ReceivePoll::Timeout { irq }
        } else if irq & IRQ_RX_DONE != 0 {
            let (mut len, start) = self.get_rx_buffer_status().await?;
            if let Some(packet) = self.packet.filter(|p| p.implicit_header) {
                len = packet.payload_len;
            }
            let status = self.get_packet_status().await?;
            let copied = buffer.len().min(usize::from(len));
            if copied != 0 {
                self.read_buffer(start, &mut buffer[..copied]).await?;
            }
            ReceivePoll::Packet {
                len,
                copied,
                status,
            }
        } else {
            ReceivePoll::Pending
        };
        self.clear_irq_status(irq).await?;
        if terminal && self.timed_rx {
            self.stop_rx().await?;
        }
        Ok(result)
    }
}
