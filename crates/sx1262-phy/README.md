# sx1262-phy

An SX1262 `no_std` driver with async `SpiDevice`, BUSY input, caller-supplied
async delay, and lifecycle/antenna-readiness hooks. Both `lora-phy` and
`lora-modulation` are pinned to reviewed
[revision b47cbdf](https://github.com/lora-rs/lora-rs/tree/b47cbdf8d3935e9bfe44c4d407bbad087fcfc179),
with default and LoRaWAN features disabled. Rust 1.88 is required for
upstream let-chains; registry publication is deferred.
Borrowed upstream backends supply modem configuration, frequency, standby,
FIFO writes, and bandwidth/IQ workarounds. Local commands retain diagnostics,
PA/OCP selection, calibration caching, managed RX and guarded timed TX.

Command references use Semtech SX1261/2 Rev 2.2, Dec 2024 (catalog
`sx1262`), §8.3.1 “BUSY Control Line” and §13 “Commands Interface”.
Upstream modem configuration also applies the §15.1 “Modulation Quality
with 500 kHz LoRa Bandwidth” and §15.4 “Optimizing the Inverted IQ
Operation” workarounds.

## Caller responsibilities

Supply an async SPI device, BUSY input and async `DelayNs`. Compose SPI/NSS
outside this crate, for example with `embedded-hal-bus::ExclusiveDevice`.
Keep synchronization at the application boundary. Check BUSY before SPI,
then wait one microsecond after NSS rises before checking BUSY again.
`BusyTiming` defaults to a 100 ms delay budget with one-ms cooperative polls.
Scheduler latency is additional: this is not a hard wall-clock deadline.

The caller owns power, reset, oscillator settling, antenna readiness and
the operating profile. Select regulator, TCXO/DIO3, DIO2 switching,
frequency, PA and OCP according to the module circuit and application.
The die's frequency limits do not describe a module's antenna bandwidth.
Digital control readback cannot establish antenna impedance or RF-path
performance. DIO2 RF switching is separate from external antenna control.

## Expander-controlled antenna

A hook can borrow a shared bus through `context`; it never stores that
borrow. Here `Controls` represents the caller's expander, power and delay
implementation. Startup establishes the antenna once. Shutdown attempts
reset, antenna-low and rail-off even after a failure. Verification reads
output mode, push-pull drive, latch and sampled level.

```rust
use sx1262_phy::{CheckPhase, CheckRequest, Hooks};

#[allow(async_fn_in_trait)]
trait Controls {
    type Error;
    async fn reset(&mut self, asserted: bool) -> Result<(), Self::Error>;
    async fn antenna(&mut self, high: bool) -> Result<(), Self::Error>;
    async fn power(&mut self, enabled: bool) -> Result<(), Self::Error>;
    async fn settle(&mut self);
    async fn confirm_antenna(&mut self, high: bool)
        -> Result<bool, Self::Error>;
}

struct ExpanderHooks;
impl<C: Controls> Hooks<C> for ExpanderHooks {
    type Error = C::Error;
    fn tx_allowed(&self) -> bool { true }
    async fn startup(&mut self, c: &mut C) -> Result<(), C::Error> {
        c.reset(true).await?;
        c.antenna(true).await?;
        c.power(true).await?;
        c.settle().await;
        c.reset(false).await?;
        c.settle().await;
        Ok(())
    }
    async fn shutdown(&mut self, c: &mut C) -> Result<(), C::Error> {
        let reset = c.reset(true).await;
        let antenna = c.antenna(false).await;
        let power = c.power(false).await;
        reset.and(antenna).and(power)
    }
    async fn verify(&mut self, c: &mut C, r: CheckRequest)
        -> Result<bool, C::Error>
    {
        c.confirm_antenna(r.phase != CheckPhase::Shutdown).await
    }
}
```

Construct with `Sx1262::with_hooks(device, busy, delay, ExpanderHooks)`.
The default `Sx1262::new` uses `DenyTx`, which denies every transmission.
`m5stack-papermono::lora::RadioHooks` implements this pattern for
PaperMono (`C153`); its board constants and operating presets stay in
the BSP. Hardware provenance: the official
[M5PaperMono-UserDemo LoRa HAL](https://github.com/m5stack/M5PaperMono-UserDemo/blob/main/main/hal/hal_lora.cpp)
and [PaperMono documentation](https://docs.m5stack.com/en/core/PaperMono).

## Permanently connected antenna

`AlwaysConnected` is an explicit zero-sized assertion that the caller
has already established power/reset and a permanent antenna connection.
Its lifecycle and confirmation hooks perform no hardware I/O.

```rust
use embedded_hal::digital::InputPin;
use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};
use sx1262_phy::{AlwaysConnected, SessionError, Sx1262};
use core::convert::Infallible;

async fn session<S: SpiDevice, B: InputPin, D: DelayNs>(
    device: S, busy: B, delay: D,
) -> Result<(), SessionError<S::Error, Infallible, B::Error>> {
    let mut radio = Sx1262::with_hooks(device, busy, delay, AlwaysConnected);
    radio.startup(&mut ()).await?;
    // Await the module's modem, oscillator and PA configuration before packets.
    radio.shutdown(&mut ()).await?;
    let (_device, _busy, _delay, _hooks) = radio.release();
    Ok(())
}
```

## Short and sustained sessions

Use `startup(context)`, explicit chip configuration, packet operations,
then `shutdown(context)` for a short diagnostic. For sustained operation,
keep the same wrapper active across RX, TX, standby and frequency
changes. Those operations never start or stop the module. An active
session preserves configuration and counters on repeated `startup`.
Restart requires shutdown followed by startup. `release` returns the SPI device,
BUSY, delay and hooks; it does not perform shutdown.

TX requires recorded active readiness and permission on every attempt.
Fresh hardware confirmation runs before attempts 1, N+1, 2N+1, and so on.
The initial interval is one. Startup and shutdown confirmations always
run, independently of that interval. `NonZeroU32` prevents a zero cadence.
Changing the interval resets its counters; selecting the same interval
does not. Packet operations retain counters. Denial, mismatch or readback
error invalidates readiness, blocks TX and restores interval one.
Further TX requires successful shutdown and startup recovery.

`set_tx(context, ticks).await` and
`write_cmd_with_context(context, CMD_SET_TX, bytes).await` share the guard.
Async `write_cmd` rejects raw `SetTx`. Both typed and raw TX require a
nonzero 24-bit hardware timeout. Unknown commands, continuous carrier/preamble
TX, chip sleep and duty-cycle RX are unsupported.

Await every command sequence to completion. Dropping an in-flight command or
lifecycle/verification future leaves `NeedsShutdown`. Recover caller-owned NSS
first, then shut down and start a new session before another TX. Successful
commands cannot repair invalidated readiness. `release` performs no I/O.
Upstream [issue 350](https://github.com/lora-rs/lora-rs/issues/350) describes
why command processing futures need care around cancellation.

`BaseBandModulationParams::new` computes LDRO from symbol duration. The wrapper
passes that value to upstream and rejects inconsistent manual overrides.
Modulation must precede packet configuration. SF5/SF6 preambles are raised to
twelve symbols. `lora_airtime_us` includes optional CRC and 16-bit preambles,
uses u64 arithmetic, and follows
[Semtech's airtime reference](https://github.com/Lora-net/sx126x_driver/blob/a10c5dfdf89788c6ac805e9fe98889de44175aa2/src/sx126x.c#L1084).
Semtech attribution and license terms accompany the package in LICENSE-Semtech.

`set_rf_frequency` caches the documented image-calibration band and recalibrates
when crossing bands; startup discards that cache. Frequencies outside documented
bands require caller-selected `calibrate_image` codes. Configure the TX clamp
with `configure_tx_clamp` after reset, retaining the selected PA/OCP settings.
Catalog `sx1262` §9.2.1 “Image Calibration for Specific Frequency Bands” and
§15.2.2 “Workaround” define these operations.

`poll_receive` rejects CRC/header failures before FIFO access and acknowledges
only observed IRQ bits during continuous RX. Caller buffers may hold a preview;
FIFO offsets wrap at 256. Timed RX completion or `stop_rx` applies the RTC
cleanup from §15.3.2 “Workaround”, preserving unrelated event-register bits.
Packet metrics retain half-dBm RSSI and quarter-dB SNR; whole-unit accessors
truncate toward zero. Repeated arrivals of the same sticky IRQ bit cannot be
distinguished by snapshots.

## Verification

Host tests cover session lifetimes, cadence one/twenty/forty, raw TX
protection, denial and recovery, rollback, shutdown evidence, BUSY/SPI/NSS
failures, parameter bounds, chip readback and upstream modem workarounds.
Pending mocks also cover yielding, cancellation, BUSY delay budgets, original
transport/upstream errors, NSS recovery, IRQ acknowledgement, timed-RX cleanup,
FIFO wrapping, fractional metrics, LDRO and airtime boundaries. These tests
establish software behavior. Settling, async timing, burst reception, RF power
and antenna performance require C153 hardware evidence.
