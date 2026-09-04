# simple-debug-fw

Blocking `esp-hal` proof-of-life image. Workspace member, **not** a
default-member: host `cargo test` must not compile this package.

First SKU: PaperMono-Lite via `m5stack-papermono-lite`.
USB-Serial/JTAG (`esp-println` `jtag-serial`), not UART0. Do not
init NFC or LoRa. No LUT. No GPIO45/46 latch (PDM). No I2C. No
Cargo `runner`.

Host-tested lines live in `crates/papermono-log`
(wire prefix `simple-debug:`):

```text
simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: git=<hash> dirty=<0|1>
simple-debug: gpio boot=1 pmic_irq=0 tp=0 ioe=1 busy=0
simple-debug: hb t=12 btn_a=1 btn_b=0
simple-debug: edge t_ms=1250 btn_a=1->0
```

Firmware emits CRLF. `hello` repeats every 10 s. Extra GPIOs are
inputs only (BOOT_OUT, PMIC IRQ, touch INT, IOE1 IRQ, EPD BUSY).

Lite (`C153-Lite`) printed those lines after `flash-app`.
Idle `gpio` with I2C and the panel off:
`boot=1 pmic_irq=0 tp=0 ioe=1 busy=0`. First CDC attach can
glue two `hb` lines; split on `simple-debug:` as well as
newline. Silicon table:
[measure.md](../../.agents/skills/m5stack-papermono-hardware/references/measure.md).

```shell
cargo xtask build-fw simple-debug
# cargo xtask flash-app --image target/xtensa-esp32s3-none-elf/release-fw/simple-debug.bin --yes --capture stock-lite
```

Live-ask, never-erase, and flash I/O: root
[AGENTS.md](../../AGENTS.md). Envelope: parent
[AGENTS.md](../AGENTS.md).

## Firmware examples as tutorial code

Firmware under `simple-debug/` serves as an educational reference
and walkthrough for bare-metal blocking `esp-hal`. Every function,
method, struct, enum, and constant (public or private) must have
comprehensive rustdoc explaining what it does, hardware nets/buses
involved, expectations, and error handling. Include abundant in-line
comments explaining hardware register sequencing, GPIO electrical
configurations (pull-ups, input modes), stack buffer usage, and
reset cycles. Ground descriptions in authoritative terminology from
*The Embedded Rust Book* and *The Rust on ESP Book*.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
