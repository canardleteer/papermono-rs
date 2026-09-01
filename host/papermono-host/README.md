# papermono-host

Programmatic host API for M5Stack PaperMono and PaperMono-Lite USB
detect, factory backup / confirm / restore, and CDC monitor
(modem lines off; `--reset` does not recapture on Lite).

`cargo xtask` is the clap front-end. Callers pass a `Layout`
(developer-data / backups root), not a hardcoded repo path.

Host-only UART grammar: `vet_idle_log` plus `papermono-log`
parse (no USB). Splits glued `simple-debug:` records. There
is no `learn-uart` CLI.

This crate does not flash unless a caller invokes restore or
`flash-app` with `--yes` and a matching per-unit snapshot. Never
`espflash flash`. Never a full-chip erase. `flash-app` writes the
snapshot `factory` partition only (Lite: `0x10000`).

`monitor` needs usbfs write access to `303a:1001`. Copy
[udev/99-papermono-usb.rules](udev/99-papermono-usb.rules) to
`/etc/udev/rules.d/`, reload udev, and replug. Operator steps:
the papermono-rs skill
[xtask.md](../../.agents/skills/papermono-rs/references/xtask.md#usbfs-udev-for-monitor).

License: MIT
