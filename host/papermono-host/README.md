# papermono-host

Programmatic host API for M5Stack PaperMono and PaperMono-Lite USB
detect, factory backup / confirm / restore, and no-reset monitor.

`cargo xtask` is the clap front-end. Callers pass a `Layout`
(developer-data / backups root), not a hardcoded repo path.

This crate does not flash unless a caller invokes restore with
`--yes` and a matching per-unit snapshot. Never `espflash flash`.
Never a full-chip erase.

`monitor` needs usbfs write access to `303a:1001`. Copy
[udev/99-papermono-usb.rules](udev/99-papermono-usb.rules) to
`/etc/udev/rules.d/`, reload udev, and replug. Operator steps:
the papermono-rs skill
[xtask.md](../../.agents/skills/papermono-rs/references/xtask.md#usbfs-udev-for-monitor).

License: MIT
