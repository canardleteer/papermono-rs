# Dependency stack

The direct-dependency inventory was checked once against crates.io on
2026-11-01, then frozen for this migration. `Cargo.lock` records the resolved
transitive versions. Use `--locked` when reproducing verification.

## Compilers

| Packages | Declared Rust minimum |
| --- | --- |
| Board/chip crates in `crates/` | 1.88 |
| `papermono-host`, `xtask`, `simple-debug-fw` | 1.95 |
| `embassy-debug-fw` | 1.97, matching pinned TrouBLE's supported compiler |

Firmware requires the Espressif Xtensa compiler, target
`xtensa-esp32s3-none-elf`. Check `rustc +esp --version`; a host stable
compiler cannot build that target. Setup is in
[getting-started.md](getting-started.md).

## Selected releases and exceptions

| Component | Selection |
| --- | --- |
| ESP HAL / runtime | `esp-hal` 1.2.2, `esp-rtos` 0.4.0 |
| ESP support | `esp-println` 0.18.0, `esp-backtrace` 0.20.0, `esp-bootloader-esp-idf` 0.6.0, `esp-alloc` 0.11.0 |
| ESP radio | **Approved exception:** published `esp-radio` 1.0.0-beta.1; stable 0.18 uses the older HAL family |
| Embassy | Executor 0.10.0, sync 0.8.0, time 0.5.1, net 0.9.1, futures 0.1.2 |
| BLE host and macros | TrouBLE commit [`866ee711c14f9d4011ca7079c5c4af0f64a1f1bb`](https://github.com/embassy-rs/trouble/tree/866ee711c14f9d4011ca7079c5c4af0f64a1f1bb), with `bt-hci` 0.11 |
| Crypto API and software backend | Embassy commit [`bd35bd7a7d19147c8bcae9c2eda99b25e597294c`](https://github.com/embassy-rs/embassy/tree/bd35bd7a7d19147c8bcae9c2eda99b25e597294c), stable `rand_chacha` 0.10.0 with default features disabled |
| LoRa | Published `lora-phy` 3.0.1 and `lora-modulation` 0.1.5; default/LoRaWAN features disabled |
| Host library | `espflash` 4.7.0, `fs4` 1.1.0, `sha2` 0.11.1 |
| Asset tools | `image` 0.25.10, `resvg` 0.49.0 |

Manifests use full commit hashes for the two approved source pins. The
lockfile also preserves Embassy crypto's upstream `unitrait` pin at
[`bb96601b3d293fa27d1ca2891f6ea148e534e433`](https://github.com/embassy-rs/unitrait/tree/bb96601b3d293fa27d1ca2891f6ea148e534e433).
Other direct dependencies use the inventoried stable releases.

## Integration contracts

Only the firmware `radio` feature enables Embassy crypto. The shared
`csprng.rs` module links the software AES-128 ECB, AES-128 CMAC and P-256
ECDH drivers and registers one Embassy RNG backed by ChaCha12. Boot
temporarily enables HAL `TrngSource` with the internal ADC, reads 32 bytes
through `Trng`, seeds ChaCha12, and drops both objects before BLE or Wi-Fi
starts. A bare HAL RNG before wireless startup is insufficient. Unseeded
crypto RNG access panics explicitly. See the selected
[HAL entropy API](https://docs.espressif.com/projects/rust/esp-hal/1.2.2/esp32s3/esp_hal/rng/index.html)
and [upstream passkey example](https://github.com/embassy-rs/trouble/blob/866ee711c14f9d4011ca7079c5c4af0f64a1f1bb/examples/esp32/src/bin/ble_bas_peripheral_pass_key.rs).
Passkey display, CDC events, GATT services, RAM-only bonds and Wi-Fi
coexistence remain part of the firmware contract.

The LoRa wrapper retains upstream SF/BW/CR/frequency validation, then
programs modulation locally so `lora-modulation`'s LDRO calculation applies
to every bandwidth without private-field access. The masked TX bandwidth
workaround preserves unrelated bits. Authority is cached catalog `sx1262`
Rev 2.2 §13.4.5.2 “LoRa Modulation Parameters” and §15.1.2 “Workaround”.
Transport faults and cancellation still invalidate readiness and require
shutdown/startup recovery. Board sequencing and TX guards are unchanged.
See [CRATES.md](CRATES.md#sx1262-upstream-comparison).

Host hashing keeps the same 64-character lowercase SHA-256 strings, so
snapshot identities, manifests, lock names and asset checksums remain
compatible. `fs4` uses its new typed nonblocking lock result; contention
still refuses immediately, inherited sessions require the matching ancestor
holder, and dropping the owning session releases the lock.

## Reproduce host verification

The repository has no GitHub Actions workflow. From the repository root,
with the Espressif environment loaded and `rumdl`, `cargo-machete` and
`cargo-audit` installed:

```shell
. "$HOME/export-esp.sh"
cargo xtask ci
cargo xtask build-fw simple-debug
cargo xtask build-fw simple-debug --no-default-features --features lite
cargo xtask build-fw embassy-debug
cargo xtask build-fw embassy-debug --no-default-features --features lite
cargo xtask build-fw embassy-debug --all-features
cargo xtask build-fw embassy-debug --no-default-features \
  --features touch,panel,orient
```

`ci` runs formatting, host Clippy/tests with default and all features,
16 firmware Clippy configurations, Markdown lint, dependency usage and
RustSec audit. Firmware packaging links crypto registrations as well as
checking source APIs. The host crypto harness compiles the exact firmware
glue and exercises uninitialized failure, seeded RNG output, AES/CMAC
vectors and P-256 agreement. LoRa tests cover all eight spreading factors,
ten bandwidths and four coding rates, masked workaround bits, validation,
transport failures and cancellation between commands. Asset encoding tests
run in a temporary directory and compare the existing bitmap and checksum.

Minimum-compiler checks and the radio-disabled dependency check:

```shell
cargo +1.88.0 check --locked -p papermono-log -p ssd1677-otp -p m5pm1 \
  -p m5ioe1 -p st25r3916 -p sx1262-phy \
  -p m5stack-papermono-lite -p m5stack-papermono
cargo +1.95.0 test --locked -p papermono-host -p xtask
cargo tree --locked -p embassy-debug-fw --target xtensa-esp32s3-none-elf \
  --no-default-features --features touch,panel,orient -e normal
```

The final tree must omit `trouble-host`, `embassy-crypto`, its software
backend and `rand_chacha`. The lockfile must contain no old P-256 0.13 /
elliptic-curve 0.13 / crypto-bigint 0.5 chain. Audit vulnerabilities must be
zero; maintenance warnings are reported separately without vulnerability
suppression.

All upgrade verification is host-only. Dated live results in the
[hardware measurement ledger](../.agents/skills/m5stack-papermono-hardware/references/measure.md)
predate this stack. Pairing/passkey rendering, coexistence, SoftAP, LoRa
recovery/sustained sessions/timings/TX power, and the ten-partial panel
policy still need appropriate physical verification per SKU. The upgrade
does not close those measurements.

## Verification record

Host-only verification on 2026-11-01 used host Rust 1.99.1 and Espressif
Rust 1.97.0.0. `cargo xtask ci` passed all 16 firmware configurations and
243 tests in each host feature configuration, formatting, Clippy, Markdown
lint across 56 files, dependency usage and audit. Rust 1.88 board checks
and 76 Rust 1.95 host tests also passed. Both local skills passed their
frontmatter validator. A coverage run passed for the changed crates.

All six image variants linked and packaged with espflash 4.7:

| Image | Features | Payload bytes |
| --- | --- | ---: |
| simple-debug | Default | 90,592 |
| simple-debug | Pruned `lite` | 90,592 |
| embassy-debug | Default | 1,119,232 |
| embassy-debug | Pruned `lite` | 1,017,088 |
| embassy-debug | All | 1,129,920 |
| embassy-debug | `touch,panel,orient`, no defaults | 167,712 |

Symbol inspection confirmed AES-128 ECB/CMAC, P-256 ECDH and exactly one
RNG registration in each radio-enabled Embassy image, with no additional
crypto drivers or unresolved crypto symbols. The simple and radio-disabled
images contained no Embassy crypto registrations. The radio-disabled tree
also omitted the new crypto dependencies.

The asset build script accepted a valid copy and rejected a stale SVG,
wrong bitmap size, missing bitmap and missing checksum in a temporary
directory. Encoding preserved the existing bitmap, inverse and checksum.
Hash vectors and lock contention/inheritance/drop regressions passed.

The final audit found **zero vulnerabilities**, removing
[RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387.html)
(`crypto-bigint` 0.5.5) and
[RUSTSEC-2026-0344](https://rustsec.org/advisories/RUSTSEC-2026-0344.html)
(`ruzstd` 0.8.2). One unsuppressed maintenance warning remains:
[RUSTSEC-2024-0449](https://rustsec.org/advisories/RUSTSEC-2024-0449.html),
transitive `unsafe-libyaml` in ESP configuration tooling. No advisory ignore
was added. No device was opened or flashed for these checks.
