# embassy-debug-fw

Embassy `esp-hal` staged image. Workspace member, **not** a
default-member: host `cargo test` must not compile this package.

First SKU: PaperMono-Lite via `m5stack-papermono-lite`.
USB-Serial/JTAG (`esp-println` `jtag-serial`), not UART0. Do
not init NFC or LoRa. No LUT. No GPIO45/46 latch (PDM). No
Cargo `runner`. CPU stays `Config::default()` (80 MHz) so
USB-Serial/JTAG PLL is unchanged.

Same CDC prefix as Path A (`simple-debug:`). Identity is
`hello image=embassy-debug`. Host-tested lines live in
`crates/papermono-log`.

Landing image: **`touch` + `panel`** (Ferris, cards, lamp).
`mic` / `radio` / `sleep` are opt-in. `simple-debug-fw` stays
featureless.

| Feature | Default | Role |
| --- | --- | --- |
| (none) | — | Async 50 ms poll, 1 Hz `hb`, 10 s `hello`/`git`/`gpio` |
| `touch` | on | I2C roster, park IP2315, FT rails, gated `charge` |
| `panel` | on | Five-card OTP walk + PWM0 lamp |
| `mic` | **off** | PDM energy + hold-A PCM dump |
| `radio` | off | `wifi n=` / `ble n=` only. No MAC/BSSID/IRK. No NVS |
| `sleep` | off | RTC/PM1 trial. Parked; not a proven 10 s wake |

`mic` and `panel` depend on `touch` (expander).

```shell
cargo xtask build-fw embassy-debug
cargo xtask build-fw embassy-debug --no-default-features
cargo xtask build-fw embassy-debug --no-default-features \
  --features touch
cargo xtask build-fw embassy-debug --features mic
cargo xtask build-fw embassy-debug --features radio
```

Do not flash `--features sleep` unless they ask.

Panel call site is `display::OtpRefresh` only:

| Card / step | Sequence |
| --- | --- |
| Tones | `GrayFull` |
| Splash / shapes / legend | `paint_mono_fast` |
| Enter targets | `MonoFull` |
| Marks | `Partial` |
| After `PARTIALS_BEFORE_FULL` (6) | next mono is `MonoFull` |

Deep sleep after each refresh; M5IOE1 `EPD_RST` to wake; no
SW reset on partial wake. Do not stamp `epd_*` or `otp_fast`.
Do not send `Partial` after `GrayFull` without `MonoFull`.
Do not start a new waveform while BUSY is high.

Five cards: splash Ferris + `papermono-rs`, shapes, legend
(A / B / red power / right lamp), four-gray tones, target
walk. Short A previous, short B next, wrap. Right-edge
contact sets PWM0 from Y (top bright). Hold A ~1 s dumps
PCM only when `mic` is on. GPIO42 chirp stays parked.

Splash art: [assets/SOURCE.md](assets/SOURCE.md). Observed
Lite glass: [docs/assets/first-ferris.png](../../docs/assets/first-ferris.png).
Silicon facts: hardware
[measure.md](../../.agents/skills/m5stack-papermono-hardware/references/measure.md)
and [display.md](../../.agents/skills/m5stack-papermono-hardware/references/display.md).

Flash only after the human is in download mode (hold red
~2 s until blink):

```shell
cargo xtask build-fw embassy-debug
# cargo xtask flash-app --image \
#   target/xtensa-esp32s3-none-elf/release-fw/embassy-debug.bin \
#   --yes --capture stock-lite
# cargo xtask monitor --for 25 --output idle-embassy.log
# cargo xtask vet-idle-log --input idle-embassy.log \
#   --image embassy-debug
```

Parent contract: [AGENTS.md](../AGENTS.md). Live-ask:
root [AGENTS.md](../../AGENTS.md).

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
