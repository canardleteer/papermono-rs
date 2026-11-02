# External skills and sources

External projects help identify code paths, design patterns, and source
conflicts. They are not this repository's measurements. Record the checked
revision when a detail matters. A project's own hardware test is evidence for
that project's reported unit and procedure; it does not automatically confirm
`C153` or `C153-Lite` here. The local evidence ledger is
[measure.md](../references/measure.md).

## FreeInk SDK

[Free-Ink/freeink-sdk](https://github.com/Free-Ink/freeink-sdk) is an active,
third-party e-reader SDK. Reviewed revision `cd6f5b83` (2026-09-30). Its
PaperMono profile is in `FREEINK_DEVICE_PAPERMONO` and
`PaperMonoBoard.h`.

Its profile describes 800×480, three-level grayscale, host-authored LUTs,
1-bit SDMMC, and M5PM1/M5IOE1 rail sequencing. Official PaperMono docs specify
480×800, four gray levels, SDMMC DAT0–DAT3, and recommend the manufacturer's
OTP refresh example because PaperMono M5GFX waveforms are currently unstable.
Keep these as source-specific statements in
[sources.md](../references/sources.md); neither the newer SDK nor its profile
closes a measurement here.

## PaperMono code examples

Repository discovery search `papermono` returned 16 repositories on
2026-10-01. These maintained or directly relevant examples were reviewed at
the listed revisions. The list records useful discovery leads, not an
endorsement of their electrical claims.

| Project | Reviewed revision | Useful coverage |
| --- | --- | --- |
| [paper_name_plate](https://github.com/ciniml/paper_name_plate/tree/7eac75dfc9a8c34509e383fd256473cf95bc5b52) | `7eac75df` (2026-09-06) | Bare-metal Rust, NFC card emulation, and BLE |
| [free-ink-on-paper-mono](https://github.com/MagicCube/free-ink-on-paper-mono/tree/34cc879427d5fc0b35f6af5084d4e7a003457665) | `34cc8794` (2026-08-27) | Lite-oriented FreeInk interface examples |
| [SeeSeeBook](https://github.com/LuoIsHere/SeeSeeBook/tree/368379ed4dd4d9920bc0e30f438c203d0f5cae17) | `368379ed` (2026-09-28) | ESP32-S3 e-reader, SD-backed books, touch, and display |
| [PaperMono shopping list](https://github.com/seamusc/papermono-shopping-list/tree/d5196df4ddf1edea0ace5ffe1aa7aab152136d6a) | `d5196df4` (2026-09-29) | Web-backed application and synchronization |
| [PaperMono Launcher](https://github.com/MingRZou/PaperMono-Launcher/tree/daf3edcee4f8669e2adf7a58bc174d62e4e40f42) | `daf3edce` (2026-08-30) | Hardware-oriented launcher work |
| [PaperMonoCalendar](https://github.com/EggUncle/PaperMonoCalendar/tree/53385729b35249c4fe4f8a3f2d7ded6ad2de5696) | `53385729` (2026-08-31) | Calendar, BLE activity tiles, clock, and RTC sync |
| [Nostos](https://github.com/osprey74/Nostos/tree/74790af8c6232df6cd22318cf38cf697e32590d5) | `74790af8` (2026-09-26) | Offline navigation and custom radio data |
| [dayring-mono](https://github.com/MagicCube/dayring-mono/tree/6e89d28140d16a8eda5878b26504fe88cbdadc23) | `6e89d281` (2026-09-22) | PaperMono application platform |
| [crossplay-papermono](https://github.com/fperuzzo72/crossplay-papermono/tree/8dc6bf2048bf7cc2d1811ec156547c5596f0604b) | `8dc6bf20` (2026-09-11) | Reader firmware bring-up |

Read source and SKU configuration before reusing a pin, power, display, or
panel-refresh claim. Similar screen dimensions and an SSD1677 part number do
not establish matching electrical wiring or safe waveforms. Generic board
guides in search results may combine PaperMono details with another board's
buttons, touch controller, RTC, or buzzer.

Search results are a dated sample, not a complete package index. Re-run the
GitHub search when refreshing this catalog. The official source catalog is
[catalog.md](../references/catalog.md).
