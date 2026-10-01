# API rules for crates written here

Derived from the Embedded Rust Book's
[HAL design checklist](https://docs.rust-embedded.org/book/design-patterns/hal/checklist.html)
and [design contracts](https://docs.rust-embedded.org/book/static-guarantees/design-contracts.html)
chapter, so "best practices" is a reviewable list rather than a
sentiment. Each rule below is auditable in review.

## Checklist

| Rule | What it means here |
| --- | --- |
| **C-CTOR** | One wrapper type per device, constructed from the buses and pins it owns. No extension traits on foreign types. |
| **C-FREE** | Every wrapper offers a destructor that consumes `self` and returns the bus and pins, leaving the device in a state where `new` can succeed again. |
| **C-HAL-TRAITS** | Implement the applicable `embedded-hal` traits when they fit. This panel crate does not ship `DrawTarget` (card UI stays in firmware). |
| **C-INLINE** | Mark small accessors `#[inline]`; cross-crate inlining is not automatic. |
| **C-PIN-STATE** | Encode device state as type parameters when a wrong sequence can hang the bus or stress the panel. |

Additional rules for this workspace:

- **No MCU dependency in a chip driver.** `m5pm1`, `m5ioe1`, and
  `ssd1677-otp` depend on `embedded-hal` only. ESP32-S3 types belong
  in the board crate or firmware.
- **Never lock a bus internally.** Take `SpiBus`, async `SpiDevice` or `I2c`
  and let the caller compose sharing.
- **`#![no_std]`, `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`.**
  Enforced through workspace lints.
- **Cite the datasheet in rustdoc** for every register, opcode, and
  magic number. Cite catalog **Id**, **section number** (when the
  sheet has one), and **section title** — not page numbers. Catalog:
  [DATASHEETS.md](DATASHEETS.md).
- **Do not invent registers.** If the datasheet has not been read,
  expose a documented raw primitive and record the gap.
- **No MCU LUT.** `ssd1677-otp` calls panel OTP only. Do not ship a
  105-byte `0x32` table.

## Typestate, and where it earns its keep

See [SAFETY.md](SAFETY.md).

- `m5ioe1`: IP2315 I2C gate (`PYG11`) is `Parked` or `Mounted`.
  `new` parks. Leaving the charger mounted can hang the bus.
- `ssd1677-otp`: `OtpRefresh::Partial` requires a mono baseline.
  `GrayFull` invalidates it. After gray, `MonoFull` before any
  `Partial`.
- M5PM1 `SYS_CMD` shutdown has an explicit method.

Keep the board crates thin: pins, SKU nets, transforms, sequencing.
They are not a second HAL over `esp-hal`.

## SX1262 lifecycle boundary

`sx1262-phy` wraps revision-pinned `lora-phy` chip operations and exposes
board-neutral caller hooks. Keep pins, power rails, antenna controls and
operating presets in the BSP. Hooks borrow context for one operation;
keep synchronization outside the chip crate. Startup/shutdown are
explicit. Packet, standby and frequency operations never control module
lifetime. Guard every TX path, including raw `SetTx`; deny by default.
Return buses, pins and hooks on release, with shutdown documented as
an explicit preceding operation. Chip I/O is async `SpiDevice` plus BUSY
and caller-supplied async delay; NSS belongs to the caller's device wrapper.
Every TX needs a nonzero hardware timeout. Await chip sequences to completion:
cancellation invalidates readiness and requires restoring NSS, shutdown and
startup before further TX. Preserve original device, BUSY and upstream errors.
Managed receive polling rejects corrupt packets before FIFO reads and clears
observed IRQ bits. Keep fractional metrics and timed-RX RTC cleanup explicit.
Registry publication stays deferred while upstream dependencies are git-pinned.

## Testing rules

- Register-level tests are `embedded-hal-mock` transaction scripts
  derived from datasheet tables.
- Sequencing tests assert **order and polarity**.
- Geometry and packing are pure functions with exact byte counts.
- Prefer a failing test that encodes a datasheet claim over a
  comment describing it.
