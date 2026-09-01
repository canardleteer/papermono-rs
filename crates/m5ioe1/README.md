# m5ioe1

Register map, GPIO bank helpers, and I2C access for the
**M5IOE1** expander. Catalog id `m5ioe1` (UM V 1.4).
`embedded-hal` 1.0 only.

`M5ioe1::new` parks `PYG11` (IP2315 I2C gate). `mount` /
`park` are typestate transitions. Which other `PYGn` nets a
board uses lives in the product BSP.

This README is the crates.io landing page. Relative markdown
links here only resolve inside this package.

## Agent notes

Portable agent rules for this crate live in `AGENTS.md` in this
directory.
