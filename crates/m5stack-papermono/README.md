# m5stack-papermono

Board support for the **M5Stack PaperMono** (`C153`).

Shared GPIOs, I2C addresses, expander, and panel types come from
`m5stack-papermono-lite`. This crate re-exports that map and adds
ST25R3916 NFC and Stamp LoRa-1262 only.

Do not depend on this crate from a PaperMono-Lite image. Lite has no
NFC or LoRa in the official HTML PinMap.

This crate does not ship a waveform LUT. It is `no_std` and
host-testable.

This README is the crates.io landing page. Relative markdown links here
only resolve inside this package.

## Agent notes

Portable agent rules for this crate live in `AGENTS.md` in this
directory.
