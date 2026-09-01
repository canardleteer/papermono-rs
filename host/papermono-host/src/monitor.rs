//! Read USB-Serial/JTAG at 115200 without pulsing DTR as download.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::Error;

/// Factory and bring-up monitor baud (reasonable first try, not measured).
pub const MONITOR_BAUD: u32 = 115_200;

/// How long to listen, how much to keep, and where to write it.
#[derive(Debug, Clone, Default)]
pub struct MonitorOptions {
    /// Stop this many seconds after the port opens. `None` means no time cap.
    pub for_secs: Option<u64>,
    /// Stop after this many newline-terminated device lines.
    pub lines: Option<u64>,
    /// Optional copy of the UART stream (tee unless [`Self::quiet`]).
    pub output: Option<PathBuf>,
    /// Write [`Self::output`] only; do not print to stdout.
    pub quiet: bool,
    /// After open, pulse USB-Serial/JTAG DTR/RTS (core reset).
    ///
    /// Lite live (2026-09-01): 0 CDC bytes; USB stayed enumerated;
    /// ACM gone; follow-up listen silent until short-press red. Not
    /// a recapture path. Off by default.
    pub reset: bool,
}

/// Copy the USB serial interface to stdout until interrupted or a budget ends.
///
/// Default listen claims the device over USB CDC so Linux `cdc-acm` never
/// opens the ACM TTY. There is no `--acm-tty` flag on this product.
pub fn monitor(port: &str, options: &MonitorOptions) -> Result<(), Error> {
    crate::detect::require_papermono_usb(port)?;
    {
        let listen = crate::cdc_listen::CdcListen::open(port)?;
        if options.reset {
            log::warn!(
                "monitor --reset: Lite live test left CDC silent; short-press red to recover"
            );
            listen.usb_jtag_serial_reset()?;
        }
        let mut reader: Box<dyn Read> = Box::new(listen);
        let mut file = match &options.output {
            Some(path) => Some(File::create(path).map_err(|error| {
                Error::Device(format!("monitor --output {}: {error}", path.display()))
            })?),
            None => None,
        };
        let mut stdout = io::stdout();
        let mut budget = ListenBudget::new(options.for_secs, options.lines);
        let mut buf = [0u8; 4096];
        loop {
            if budget.is_exhausted() || crate::cdc_listen::interrupt_requested() {
                break;
            }
            match reader.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    emit(&mut stdout, file.as_mut(), options.quiet, &buf[..n])?;
                    budget.note_bytes(&buf[..n]);
                }
                Err(error) if error.kind() == io::ErrorKind::TimedOut => {}
                Err(error) if error.kind() == io::ErrorKind::Interrupted => break,
                Err(error) => return Err(Error::Device(format!("UART read failed: {error}"))),
            }
        }
    }
    if !crate::cdc_listen::wait_for_kernel_tty(Duration::from_secs(2)) {
        log::warn!(
            "cdc-acm did not reappear; unplug/replug if the next command cannot see the TTY"
        );
    }
    Ok(())
}

fn emit(
    stdout: &mut impl Write,
    file: Option<&mut File>,
    quiet: bool,
    bytes: &[u8],
) -> Result<(), Error> {
    if !quiet {
        stdout.write_all(bytes)?;
        stdout.flush()?;
    }
    if let Some(file) = file {
        file.write_all(bytes)?;
        file.flush()?;
    }
    Ok(())
}

pub(crate) struct ListenBudget {
    deadline: Option<Instant>,
    max_lines: Option<u64>,
    lines_seen: u64,
}

impl ListenBudget {
    fn new(for_secs: Option<u64>, max_lines: Option<u64>) -> Self {
        Self {
            deadline: for_secs.map(|secs| Instant::now() + Duration::from_secs(secs)),
            max_lines,
            lines_seen: 0,
        }
    }

    fn note_bytes(&mut self, bytes: &[u8]) {
        self.lines_seen = self
            .lines_seen
            .saturating_add(bytes.iter().filter(|b| **b == b'\n').count() as u64);
    }

    fn is_exhausted(&self) -> bool {
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return true;
        }
        matches!(self.max_lines, Some(max) if self.lines_seen >= max)
    }
}

#[cfg(test)]
mod tests {
    use super::{ListenBudget, MONITOR_BAUD};

    #[test]
    fn baud_is_first_try() {
        assert_eq!(MONITOR_BAUD, 115_200);
    }

    #[test]
    fn a_line_cap_counts_newlines_across_chunks() {
        let mut budget = ListenBudget::new(None, Some(3));
        budget.note_bytes(b"ab\ncd");
        assert!(!budget.is_exhausted());
        budget.note_bytes(b"\nef\n");
        assert!(budget.is_exhausted());
    }
}
