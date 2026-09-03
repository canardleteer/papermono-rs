# simple-debug-fw

Blocking `esp-hal` proof-of-life image for PaperMono-Lite. No
Embassy, no panel refresh, no I2C. Host-tested line format:
[`crates/papermono-log`](../../crates/papermono-log)
(wire prefix `simple-debug:`).

```text
simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: git=<hash> dirty=<0|1>
simple-debug: gpio boot=1 pmic_irq=0 tp=0 ioe=1 busy=0
simple-debug: hb t=12 btn_a=1 btn_b=0
simple-debug: edge t_ms=1250 btn_a=1->0
```

On the unit:

- Repeating `hello` / `git` / `gpio` every 10 s, and a 1 Hz
  `hb`. The glass does not change.
- BUTTON A (UP) and BUTTON B (DOWN) print `edge` on press and
  release. `btn_a=1` / `btn_b=1` is idle (released).
- Output streams over the native USB-Serial/JTAG interface.

First-time toolchain and snapshot:
[docs/getting-started.md](../../docs/getting-started.md).

## Heartbeat check

### Step 1: Is the port free?

Only one `monitor` at a time. Ctrl-C an old listen. Do not
`kill -9`.

```shell
cargo xtask detect-connected
```

You should see an Espressif `303a:1001` path. If you do not,
and you already killed a listen the hard way, unplug USB-C and
plug it back in once.

### Step 2: Build and flash

Hold the red power button about 2 s until it blinks
(download), then:

```shell
. $HOME/export-esp.sh
cargo xtask build-fw simple-debug
cargo xtask flash-app \
  --image target/xtensa-esp32s3-none-elf/release-fw/simple-debug.bin \
  --yes
```

If your backup was a named capture (`--name SLUG`) instead of
`--as-original`, pass `--capture SLUG`. Short-press red to leave
the bootloader, then:

```shell
cargo xtask monitor
```

Ctrl-C when you are done. Do not `kill -9`.

### Step 3: What you should see

```text
simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: hb t=12 btn_a=1 btn_b=1
```

`hello` repeats every 10 s so a late attach still sees
identity. First CDC attach can glue two lines; look for the
`simple-debug:` prefix.

### Step 4: Press the keys

Press BUTTON A, then BUTTON B. You should see `edge` lines
(`1->0` on press, `0->1` on release) and `hb` with
`btn_a=` / `btn_b=` matching the held key.

Host check on a quiet capture:
`cargo xtask vet-idle-log --input idle-simple.log`.

Agent flash contract: [AGENTS.md](AGENTS.md).
