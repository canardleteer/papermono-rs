# Rust software paths

Hardware stays in
[`m5stack-papermono-hardware`](../../m5stack-papermono-hardware/SKILL.md).
This page is how to drive that hardware from **Rust** in this
repository. Command catalog: [xtask.md](xtask.md). Crate rules:
[`docs/API-RULES.md`](../../../../docs/API-RULES.md). Verdicts:
[`docs/CRATES.md`](../../../../docs/CRATES.md).

Two stacks are valid. In-tree images are **`no_std` / `esp-hal`**:

| Stack | When |
| --- | --- |
| `no_std`: `esp-hal` + `esp-rtos` / Embassy | Bare-metal async; this repo’s default |
| `std`: `esp-idf-hal` + `esp-idf-svc` | Share ESP-IDF drivers with vendor C++ |

Encode the
[pin-map](../../m5stack-papermono-hardware/references/pin-map.md)
in `m5stack-papermono-lite` / `m5stack-papermono`. Chip drivers
(`m5pm1`, `m5ioe1`, `ssd1677-otp`) stay MCU-agnostic. Adopt a
crates.io driver only with a recorded verdict in
[`docs/CRATES.md`](../../../../docs/CRATES.md).

Do not mix this page with PlatformIO / `idf.py`. Those trees are
wiring evidence in
[cpp-platformio.md](../../m5stack-papermono-hardware/references/cpp-platformio.md).
USB/flash:
[flashing.md](../../m5stack-papermono-hardware/references/flashing.md).
Observed silicon:
[measure.md](../../m5stack-papermono-hardware/references/measure.md).

## Host toolchain (`cargo xtask`)

USB-C is Espressif USB-Serial/JTAG (`303a:1001`), not a QinHeng
CH343. Download is a power-button hold (~2 s until the red LED
blinks). `cargo xtask` refuses `1a86:55d3` **before** opening a
port. Full catalog and USB lock: [xtask.md](xtask.md).

`papermono-host` talks through the **`espflash` library**. Never
`espflash flash`. Never `erase-flash`. There is no Cargo `runner`.

Do not open a port unless a human asked.

### One-time install

```shell
cargo install espup --locked
espup install
# then source the script `espup` printed (example: . $HOME/export-esp.sh)
```

`espup` provides the Xtensa compiler. `build-fw` runs host-only
`save-image --flash-size 16mb`. The host user needs `dialout`.
`monitor` also needs the
[usbfs udev rule](xtask.md#usbfs-udev-for-monitor).

### `no_std` (`esp-hal`) — Cargo

Target: `xtensa-esp32s3-none-elf`.

```shell
. $HOME/export-esp.sh
cargo xtask build-fw simple-debug
cargo xtask build-fw embassy-debug
```

`save-image` refuses an `esp-hal` ELF without
`esp_bootloader_esp_idf::esp_app_desc!()`. Do not `--merge`.
Lite factory is `0x10000`. After `flash-app`, short-press red.

Ship a **16 MB-aware** table. Do not assume arbitrary partition
offsets or 32 MB geometry.

## Datasheet catalog vs crates

| Catalog id | Crate | Notes |
| --- | --- | --- |
| `ssd1677` | `ssd1677-otp` | OTP-Demo `OtpRefresh`. No `0x32` LUT |
| `m5pm1` | `m5pm1` | Registers + PWM0. Board nets stay in the BSP |
| `m5ioe1` | `m5ioe1` | GPIO banks + IP2315 gate typestate |
| `ft6336g` | BSP `touch` | Public PDF has no map. M5GFX decode only |
| `bmi270` | BSP `imu` | `CHIP_ID` only so far |
| `ip2315` | BSP / firmware | Park via `m5ioe1` except a gated sit |

## Crates vs parts

| Part | Crate | Notes |
| --- | --- | --- |
| Shared pins / SKU | `m5stack-papermono-lite` | This repo. Lite firmware depends on this only |
| C153 radios | `m5stack-papermono` | NFC + LoRa add-on |
| SSD1677 OTP | `ssd1677-otp` | Not crates.io `ssd1677`. Dedicated panel OTP driver |
| M5PM1 | `m5pm1` | PWM0 is G3. PWM1 is unused on this SKU |
| M5IOE1 | `m5ioe1` | Board `0x4F`. Park IP2315 on `PYG11` |
| CDC lines | `papermono-log` | Host-tested grammar for both images |

## Wi-Fi / BLE in embassy-debug

Landing image defaults `--features radio`. Cards: BLE
passkey (`PaperMono`), Wi-Fi channel survey, WPA2 SoftAP
(`PaperMono-AP` / `mono2026`, `192.168.4.1`, DHCP + JSON
HTTP). Survey ↔ SoftAP mutually exclusive. Soft status
redraws use OTP Partial past the usual budget; card change
still honors `PARTIALS_BEFORE_FULL`. Agent workflows:
[firmware/embassy-debug/AGENTS.md](../../../../firmware/embassy-debug/AGENTS.md).
Crate verdicts: [`docs/CRATES.md`](../../../../docs/CRATES.md)
(**Radio**). Silicon: hardware
[measure.md](../../m5stack-papermono-hardware/references/measure.md).

Opt-in `--features orient`: BMI270 page rotation (sticky-rs
policy). Lite axis map: USB-C down = −X (see sensors.md).
Same-card remaps soft-Partial; nav arms on button release
with 3-sample IMU hysteresis
([embassy-debug/AGENTS.md](../../../../firmware/embassy-debug/AGENTS.md)).
