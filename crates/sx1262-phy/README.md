# sx1262-phy

An SX1262 `no_std` driver using blocking `embedded-hal` 1.0 SPI/GPIO,
with caller-supplied asynchronous lifecycle and antenna-readiness hooks.
It wraps the published `lora-phy` SX126x driver for modem configuration,
frequency, standby, FIFO writes and packet TX. `lora-modulation` supplies
LoRa types and symbol timing. Local commands cover diagnostic readback,
explicit PA/OCP settings, register access, IRQs and bounded RX/TX timers.

Command references use Semtech SX1261/2 Rev 2.2, Dec 2024 (catalog
`sx1262`), §8.3.1 “BUSY Control Line” and §13 “Commands Interface”.
Upstream modem configuration also applies the §15.1 “Modulation Quality
with 500 kHz LoRa Bandwidth” and §15.4 “Optimizing the Inverted IQ
Operation” workarounds.

## Caller responsibilities

Supply a dedicated SPI bus, initially high NSS, and a BUSY input.
BUSY polling has an iteration budget. Its duration depends on CPU speed.
SPI transactions block; hooks may await without requiring an executor
inside this crate. Keep synchronization at the application boundary.

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

Construct with `Sx1262::with_hooks(spi, nss, busy, ExpanderHooks)`.
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
use embedded_hal::{digital::{InputPin, OutputPin}, spi::SpiBus};
use sx1262_phy::{AlwaysConnected, SessionError, Sx1262};
use core::convert::Infallible;

async fn session<S: SpiBus, N: OutputPin, B: InputPin>(
    spi: S, nss: N, busy: B,
) -> Result<(), SessionError<S::Error, Infallible>> {
    let mut radio = Sx1262::with_hooks(spi, nss, busy, AlwaysConnected);
    radio.startup(&mut ()).await?;
    // Configure the module's modem, oscillator and PA before packet operations.
    radio.shutdown(&mut ()).await?;
    let (_spi, _nss, _busy, _hooks) = radio.release();
    Ok(())
}
```

## Short and sustained sessions

Use `startup(context)`, explicit chip configuration, packet operations,
then `shutdown(context)` for a short diagnostic. For sustained operation,
keep the same wrapper active across RX, TX, standby and frequency
changes. Those operations never start or stop the module. An active
session preserves configuration and counters on repeated `startup`.
Restart requires shutdown followed by startup. `release` returns SPI,
NSS, BUSY and hooks; it does not perform shutdown.

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
Blocking `write_cmd` rejects raw `SetTx`. Unknown commands, continuous
carrier/preamble TX, chip sleep and duty-cycle RX are unsupported.
Do not cancel lifecycle or verification futures: cancellation leaves
`NeedsShutdown`, requiring explicit cleanup before another startup.

`BaseBandModulationParams::new` selects LDRO from symbol duration. The
published upstream driver owns its LDRO choice; inconsistent manual
LDRO overrides are rejected. Modulation must be configured before
packet parameters. Upstream raises SF5/SF6 preambles to twelve symbols.
The caller must select a matching peer profile. General GFSK packet
configuration is outside this wrapper's LoRa parameter API.

## Verification

Host tests cover session lifetimes, cadence one/twenty/forty, raw TX
protection, denial and recovery, rollback, shutdown evidence, BUSY/SPI/NSS
failures, parameter bounds, chip readback and upstream modem workarounds.
These tests establish software behavior. RF validation requires hardware.
