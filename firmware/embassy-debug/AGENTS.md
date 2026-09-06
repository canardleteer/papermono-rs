# embassy-debug-fw

Embassy `esp-hal` staged image. Workspace member, **not** a
default-member: host `cargo test` must not compile this package.

Primary baseline: PaperMono-Lite via `m5stack-papermono-lite`.
Safe across both SKUs for shared peripherals (display, touch, buttons,
frontlight, buzzer, IMU, battery gauge, BLE/Wi-Fi). On PaperMono (`C153`),
NFC and LoRa pins remain unasserted inputs / quiescent in standard builds.
USB-Serial/JTAG (`esp-println` `jtag-serial`), not UART0. Do
not init NFC or LoRa in default images. No LUT. No GPIO45/46 latch (PDM). No
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
| `orient` | on | BMI270 page rotation (sticky-rs style). Lite axis map glass-confirmed 2026-09-04 |

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

Eleven cards: splash Ferris + `papermono-rs`, lora_scan
(US915 LoRa channel energy sweeper + packet detection
across 104 slots; touch `[ START SCAN ]`), lora (Stamp
LoRa-1262 transceiver test ping + up to 60 s packet sniffer;
touch `[ TX PING ]` / `[ LISTEN RX ]`), nfc (ST25R3916 near
field communication tag poll; touch `[ POLL TAG ]`), wifi_ap
(WPA2 SoftAP `PaperMono-AP` / `mono2026` + DHCP + JSON HTTP at
`http://192.168.4.1/`; touch `[ START HOTSPOT ]`; mutually
exclusive with survey), wifi_survey (2.4 GHz channel occupancy
and top APs; touch `[ START SURVEY ]`), bluetooth (BLE peripheral
pairing with 6-digit passkey display and success/fail reason),
legend (A / B / sleep / red power / right lamp / live battery %
gauge with 60 s auto-refresh), shapes (procedural 3-degree Koch
snowflake with microsecond benchmark), four-gray tones, target
walk. Short A previous, short B next (down-press); button
presses during EPD paint are queued so clicks are never
dropped; wrap. Right-edge contact sets PWM0 from Y (top bright);
left-edge contact sets buzzer volume from Y (top loud); both
sliders respect screen orientation. Hold A 2 s triggers sleep
notice and light sleep; hold A or B 1 s wakes. Hold A ~1 s dumps
PCM only when `mic` is on. GPIO42 passive buzzer provides click
feedback on button navigation and touchscreen hits. Active channel
survey runs continuously until touching `[ STOP SURVEY ]`
(transitions through `[ STOPPING... ]` to complete).

## Carousel Order Guidance

`Splash` is the landing card (index 0). The forward walk
(`next()` / BUTTON B / down-press) displays newer interactive
feature cards first in reverse chronological order of feature
introduction (`LoraScan` → `Lora` → `Nfc` → `WifiAp` →
`WifiSurvey` → `Bluetooth` → `Legend` → `Shapes`), followed by
baseline display calibration cards (`Tones` → `Targets`),
wrapping back to `Splash`. When developing or validating new
firmware features, place newly added interactive test cards
immediately after `Splash` in the forward walk. This minimizes
operator button presses needed to reach active development tests
on hardware.

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

## LoRa verification workflow (`c153`)

On `C153`, the Stamp LoRa-1262 (Semtech SX1262) transceiver is
exercised via two interactive cards:

1. **LoRa Sweeper (`lora_scan`)**:
   - Sweeps across all 104 US915 channels (902.125 MHz to 927.875 MHz).
   - Touch `[ START SCAN ]` to begin continuous sweeping passes (~2.5 s/pass).
   - Applies double-duty scanning with doubled dwell time (25 ms vs 10 ms)
     on expected primary channels (slots 61..=63 and slot 19).
   - Generates acoustic tone feedback via GPIO42 passive buzzer: a 25 ms
     click on elevated RSSI activity (above -105 dBm) and an 80 ms tone on
     packet detection.
   - Emits `simple-debug: lora_scan slot=... freq=... rssi=... packets=...`
     telemetry at the completion of each pass over CDC.
   - Touch anywhere on the panel to stop sweeping (`[ STOP SCAN ]`).

2. **LoRa Transceiver (`lora`)**:
   - Touch `[ TX PING ]` for a bench-safe test transmission: 915.0 MHz,
     clamped to +14 dBm (25 mW), 60 mA hardware OCP current limit, 58 ms
     airtime, with immediate return to Standby RC and power-down. Emits
     `lora_tx` telemetry over CDC.
   - Touch `[ LISTEN RX ]` for an extended packet sniffer window (up to 60 s).
     Listens on 906.875 MHz (US915 slot 19) or secondary channel using
     SF11 / BW 250 kHz / CR 4/5 / Sync Word `0x24B4`. Emits an 80 ms tone
     and logs `lora_rx freq=... rssi=... snr=... len=... preview=...` over
     CDC upon receiving a packet. Touch or buttons abort early.

## NFC verification workflow (`c153`)

On `C153`, the ST25R3916 near-field communication controller is
exercised via the `nfc` card:

- Touch `[ POLL TAG ]` to initiate ISO14443-A polling.
- Powers the chip via M5IOE1 `PYG4`, confirms 3.3 V supply configuration,
  waits for crystal oscillator stability, and energizes the 13.56 MHz RF field.
- Transmits 7-bit short-frame WUPA/REQA commands, performs anticollision
  cascades (CL1 / CL2), reads SAK, and extracts the card UID.
- Masks middle UID bytes on serial for privacy (`nfc_tag type=iso14443a`).
- Renders tag details (UID, SAK, cascade level) on the e-paper panel.
- Immediately de-energizes the RF field and parks the peripheral between
  polls to conserve power and prevent bus contention.

## IMU page rotation (`orient`)

Default Cargo feature. Sticky-rs policy ported to BMI270:

- Dominant-axis classify at 0.70 g; FaceUp/FaceDown keep last page.
- Draw in page space (`PageRotation`); map via
  `display::page_to_framebuffer` into fixed USB-down 480×800 planes.
- Touch Wi-Fi buttons use `framebuffer_to_page` then `draw::wifi_action_hit`.
  Volume (left edge) and frontlight (right edge) gutters are also evaluated in
  page space across all four orientations.
- CDC `imu pose=… x=… y=… z=…` every 5 s and on page change.
- Axis→pose (Lite 2026-09-04): −X `Portrait0`, +X `Portrait180`,
  +Y `Landscape0`, −Y `Landscape180`. `C153` still unconfirmed.
- Same-card remaps are **soft** `Partial` (no `MonoFull` budget hit).
- Bring-up: Bosch standard **8 KiB** `bmi270_config_file` with
  `INIT_ADDR_*` chunking; `INTERNAL_STATUS` is `0x21`. Do **not**
  use the maximum-FIFO config blob (wrong variant; XYZ stay zero).
- Nav: monitor buttons concurrently during EPD paint to queue clicks;
  handle buttons before IMU; require `IMU_STABLE_POLLS` (3) agreeing
  samples before remapping.

```shell
cargo xtask build-fw embassy-debug
```

## Card navigation edges

Button monitoring runs concurrently during long EPD paints
(`paint_with_buttons` via `select`), queuing navigation events so rapid clicks
are never dropped. Button A short-press fires on release (distinguishing
long-press sleep); Button B fires immediately on press-down (falling edge).
Audible key clicks sound on every valid button press. Soft orientation / radio
refreshes run after button handling so a same-tick press wins.

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
