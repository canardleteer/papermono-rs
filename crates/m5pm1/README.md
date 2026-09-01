# m5pm1

Register map and I2C access for the **M5PM1** PMIC. Catalog id
`m5pm1` (UM V 1.9). `embedded-hal` 1.0 only.

PWM0 is the multiplexed engine on GPIO3. PWM1 is a different
timer and is not this crate’s frontlight path. Board nets
(which `Gn` is the lamp) live in the product BSP.

This README is the crates.io landing page. Relative markdown
links here only resolve inside this package.

## Agent notes

Portable agent rules for this crate live in `AGENTS.md` in this
directory.
