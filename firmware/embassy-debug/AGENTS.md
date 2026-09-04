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

Landing image: **`touch` + `panel` + `sleep` + `radio`** (Ferris,
cards, lamp, sleep, BLE pairing, Wi-Fi survey, SoftAP). `mic` is
opt-in. `simple-debug-fw` stays featureless.

| Feature | Default | Role |
| --- | --- | --- |
| (none) | — | Async 50 ms poll, 1 Hz `hb`, 10 s `hello`/`git`/`gpio` |
| `touch` | on | I2C roster, park IP2315, FT rails, gated `charge` |
| `panel` | on | Eight-card OTP walk + PWM0 lamp |
| `mic` | **off** | PDM energy + hold-A PCM dump |
| `radio` | on | BLE pairing + Wi-Fi survey / SoftAP cards + wifi/ble counts. No MAC/BSSID/IRK. No NVS |
| `sleep` | on | Button A hold 2 s to sleep, 1 s A/B hold to wake |
| `orient` | **off** | BMI270 page rotation (sticky-rs style). Provisional axis map until glass-confirmed |

`mic`, `panel`, `sleep`, and `orient` depend on `touch`
(expander). `sleep` / `orient` depend on `panel`.

```shell
cargo xtask build-fw embassy-debug
cargo xtask build-fw embassy-debug --no-default-features
cargo xtask build-fw embassy-debug --no-default-features \
  --features touch
cargo xtask build-fw embassy-debug --features mic
cargo xtask build-fw embassy-debug --features radio
cargo xtask build-fw embassy-debug --features orient
```

Panel call site is `display::OtpRefresh` only:

| Card / step | Sequence |
| --- | --- |
| Tones | `GrayFull` |
| Splash / shapes / legend / bluetooth / wifi_survey / wifi_ap | `paint_mono_fast` (`soft` on same-card Bluetooth / Wi-Fi / Legend status redraws **and** same-card orientation remaps → stay on `Partial`; card change honors partial budget → `MonoFull`) |
| Enter targets | `MonoFull` |
| Marks | `Partial` |
| After `PARTIALS_BEFORE_FULL` (18) | next **non-soft** mono is `MonoFull` |

Deep sleep after each refresh; M5IOE1 `EPD_RST` to wake; no
SW reset on partial wake. Do not stamp `epd_*` or `otp_fast`.
Do not send `Partial` after `GrayFull` without `MonoFull`.
Do not start a new waveform while BUSY is high.

Eight cards: splash Ferris + `papermono-rs`, shapes (procedural
3-degree Koch snowflake with microsecond benchmark), legend
(A / B / sleep / red power / right lamp / live battery % gauge with
60 s auto-refresh), bluetooth (BLE peripheral pairing with 6-digit
passkey display and success/fail reason), wifi_survey (2.4 GHz
channel occupancy + top APs; touch `[ START SURVEY ]`), wifi_ap
(WPA2 SoftAP `PaperMono-AP` / `mono2026` + DHCP + JSON HTTP at
`http://192.168.4.1/`; touch `[ START HOTSPOT ]`; mutually
exclusive with survey), four-gray tones, target walk. Short A
previous, short B next, wrap. Right-edge contact sets PWM0 from Y
(top bright). Hold A 2 s triggers sleep notice and light sleep;
hold A or B 1 s wakes. Hold A ~1 s dumps PCM only when `mic` is on.
GPIO42 chirp stays parked.

Splash art: [assets/SOURCE.md](assets/SOURCE.md). Observed
Lite glass: [docs/assets/first-ferris.png](../../docs/assets/first-ferris.png),
[docs/assets/koch-snowflake.png](../../docs/assets/koch-snowflake.png),
[docs/assets/battery.png](../../docs/assets/battery.png),
and [docs/assets/pm-wifi.png](../../docs/assets/pm-wifi.png).
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

## Bluetooth pairing verification workflow

When testing the `bluetooth` card pairing functionality, two
verification pathways are supported:

1. **Manual external device pairing**: The human navigates to the
   `bluetooth` card, searches for `PaperMono` from their phone or
   central device, initiates pairing, reads the 6-digit numeric passkey
   rendered on the e-paper glass, enters it into the phone, and observes
   the success banner on the display.
2. **Host-agent self-diagnostic pairing (faster for agents)**: If the
   host system has an available, unblocked Bluetooth controller (e.g.
   via BlueZ `bluetoothctl`), the agent can perform an automated
   self-diagnostic when explicitly requested by the user:
   - Monitor the device CDC stream (`cargo xtask monitor`) to capture
     live pairing events.
   - Scan and discover `PaperMono` (`F1:33:22:11:42:F5`).
   - Initiate pairing to provoke the passkey exchange.
   - Extract the 6-digit passkey from the CDC stream (`pair pin=XXXXXX`).
   - Submit the extracted PIN to `bluetoothctl` to complete bonding and
     verify `pair ok`.
   - Ask the user to visually confirm that the same PIN and success
     banner appeared on the e-paper glass.

Always offer the human the option to test with their own devices; the
automated host pathway provides fast, reproducible self-diagnostics when
available.

## Wi-Fi survey and SoftAP verification workflow

Survey and SoftAP are mutually exclusive: starting one stops the
other. One Wi-Fi manager task owns the mode machine (`Idle` /
`SurveyScanning` / `SurveyComplete` / `Hotspot`). Stack under
`radio`: `esp-radio` (STA scan + SoftAP), `embassy-net`,
`edge-dhcp`, `edge-nal` / `edge-nal-embassy` (DHCP + HTTP).

| Constant | Value |
| --- | --- |
| SSID | `PaperMono-AP` |
| Auth | WPA2-Personal only (`mono2026`) |
| Gateway | `192.168.4.1/24` |
| HTTP | `GET /` JSON on port 80 (`sku`, battery, `wifi.clients`, `wifi.requests`) |
| Survey | Channels 1–13; top **4** APs by RSSI on glass; CDC counts only |

WPA3/SAE is unavailable in the precompiled `esp-radio` ESP32-S3
wireless blob — do not advertise WPA3 on glass. No foreign
MAC/BSSID/IRK on CDC. SoftAP CDC may print the fixed demo
SSID/password. Survey glass may show truncated nearby SSIDs;
do not echo those on the wire.

**UI width:** portrait is 480 px wide. Guide lines use
`FONT_10X20` at `x = 40` (~10 px/glyph) → stay at about **40
glyphs** or they clip the right edge.

**Soft refresh:** same-card Bluetooth / Wi-Fi / Legend redraws
and same-card orientation remaps use
`paint_mono_fast(..., soft = true)` so they stay on OTP
`Partial` past `PARTIALS_BEFORE_FULL` instead of flashing
`MonoFull`. Card change honors the budget.

1. **Channel survey**: Human opens the `wifi_survey` card and taps
   `[ START SURVEY ]`. CDC prints `wifi_survey count=… ch1=… ch6=… ch11=…`.
   Glass shows channel occupancy and the strongest APs. Starting survey
   tears down an active SoftAP.
2. **SoftAP + JSON HTTP**: Human opens the `wifi_ap` card and taps
   `[ START HOTSPOT ]`. Glass shows SSID, segmented password, URL, client
   count, and HTTP request count. CDC prints `wifi_ap state=active …`.
3. **Host-agent SoftAP check** (when the human asks for a live test and a
   spare host Wi-Fi adapter is available, e.g. `wlx9cefd5f6363b`):
   - Monitor CDC (`cargo xtask monitor`) for `wifi_ap` / `wifi_http`.
   - Scan: `nmcli dev wifi list ifname IFACE`.
   - Connect: `nmcli dev wifi connect PaperMono-AP password mono2026 ifname IFACE`.
   - Fetch: `curl -s http://192.168.4.1/` and confirm JSON battery / Wi-Fi
     fields; CDC should print `wifi_http`.
   - Disconnect and confirm client count drops on glass / CDC.
   - First DHCP lease observed on Lite: `192.168.4.50`.

Lite SoftAP host-verified 2026-09-04:
[measure.md](../../.agents/skills/m5stack-papermono-hardware/references/measure.md).
`C153` still open.

## IMU page rotation (`orient`)

Opt-in Cargo feature. Sticky-rs policy ported to BMI270:

- Dominant-axis classify at 0.70 g; FaceUp/FaceDown keep last page.
- Draw in page space (`PageRotation`); map via
  `display::page_to_framebuffer` into fixed USB-down 480×800 planes.
- Touch Wi-Fi buttons use `framebuffer_to_page` then
  `draw::wifi_action_hit`. Lamp gutter stays **physical** right edge.
- CDC `imu pose=… x=… y=… z=…` every 5 s and on page change.
- Axis→pose (Lite 2026-09-04): −X `Portrait0`, +X `Portrait180`,
  +Y `Landscape0`, −Y `Landscape180`. `C153` still unconfirmed.

```shell
cargo xtask build-fw embassy-debug --features orient
```

## Firmware examples as tutorial code

Firmware under `embassy-debug/` serves as an educational reference
and walkthrough for async Embassy on ESP32-S3. Every function,
method, struct, enum, and constant (public or private) must have
comprehensive rustdoc explaining what it does, hardware nets/buses
involved, expectations, and error handling. Include abundant in-line
comments explaining hardware register sequencing, GPIO electrical
configurations (pull-ups, input modes), bus arbitration, Embassy task
scheduling, stack buffer usage, and reset/wake-up cycles. Ground
descriptions in authoritative terminology from *The Embedded Rust Book*,
*The Rust on ESP Book*, and *The Embassy Book*.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
