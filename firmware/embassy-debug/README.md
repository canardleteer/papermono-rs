# embassy-debug-fw

Embassy staged image for PaperMono-Lite. USB-Serial/JTAG prints
the same `simple-debug:` lines as the proof-of-life image, with
`hello image=embassy-debug`. Host-tested line format:
[`crates/papermono-log`](../../crates/papermono-log)
(wire prefix `simple-debug:`).

On the unit:

- Cold boot paints a 1-bit Ferris splash and `papermono-rs`
  (USB-C down). Original Ferris: Karen Rustad Tölva
  ([rustacean.net](https://rustacean.net/)). Line-art
  monification details in [assets/SOURCE.md](assets/SOURCE.md).
- BUTTON A previous card, BUTTON B next. The walk is splash →
  shapes → legend → four-gray tones → touch targets.
- Slide the right edge for the lamp (top bright, USB-C dim).
- Hold BUTTON A about 2 s to enter low-power sleep; hold BUTTON A
  or B about 1 s to wake up.
- Hold BUTTON A about 1 s for a PCM dump when the image
  was built `--features mic` (default image leaves mic off).
- The GPIO42 buzzer chirp is parked.

Default features are `touch`, `panel`, and `sleep`. The `mic` and
`radio` features are opt-in. First SKU is Lite.

First-time toolchain and snapshot:
[docs/getting-started.md](../../docs/getting-started.md).

## Card walk and lamp

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
cargo xtask build-fw embassy-debug
cargo xtask flash-app \
  --image target/xtensa-esp32s3-none-elf/release-fw/embassy-debug.bin \
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

Glass: Ferris, then `papermono-rs`, then the button / lamp
hint. CDC:

```text
simple-debug: hello t=0 image=embassy-debug sku=C153-Lite …
simple-debug: scene=splash
simple-debug: hb t=12 btn_a=1 btn_b=1
```

`hello` repeats every 10 s. Host check on a quiet capture:
`cargo xtask vet-idle-log --input idle-embassy.log --image
embassy-debug`.

### Step 4: Change cards

Short-press BUTTON B. The glass should walk splash → shapes →
legend → tones → targets, then wrap. BUTTON A walks the other
way. CDC prints `scene=`.

### Step 5: Slide the lamp

On any card, slide a finger along the right edge. Top of the
glass is brighter; USB-C is dimmer. CDC prints `lamp=`.

### Step 6: Sleep and wake

On any card, hold BUTTON A about 2 s. The frontlight and red power LED turn
off, and the glass displays "sleeping, press A or B for 1 second to restart".
Hold BUTTON A or B about 1 s to wake up; the lamp and red LED turn back on and
the prior card is restored.

Agent flash contract: [AGENTS.md](AGENTS.md).
