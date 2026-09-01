# papermono-log

Host-tested USB-Serial/JTAG line format for
`firmware/simple-debug` (`simple-debug-fw`) **and**
`firmware/embassy-debug` (`embassy-debug-fw`,
`hello image=embassy-debug`). This crate is a
default-member. The Xtensa images are not: do not
`cargo test -p simple-debug-fw` or
`cargo test -p embassy-debug-fw` on host rustc.

The **wire prefix** stays `simple-debug:`. Only the crate
directory and package name changed.

Flash, live-ask, and envelope:
[firmware AGENTS.md](../../firmware/AGENTS.md),
[simple-debug-fw](../../firmware/simple-debug/AGENTS.md),
[embassy-debug-fw](../../firmware/embassy-debug/AGENTS.md),
and the root [AGENTS.md](../../AGENTS.md).

```shell
cargo test -p papermono-log --locked
```

This crate’s `README.md` is the crates.io landing page. Relative
markdown links there only resolve inside this package.

## Agent Documentation Standards

Maintain this file according to the [AGENTS.md standard](https://agents.md/),
and keep it portable across compatible agent clients, without
assumptions about user-specific paths or session state.
