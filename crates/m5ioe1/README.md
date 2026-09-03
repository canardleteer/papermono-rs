# m5ioe1

Register map, GPIO bank helpers, and I2C communication for the M5Stack
M5IOE1 I/O expander.

Datasheet: [M5Stack IO Expander Datasheet V 1.4](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1210/IO_Expander_Datasheet_EN.pdf).
Further product information is available through
[M5Stack Documentation](https://docs.m5stack.com).

This crate provides a `#![no_std]` driver based on `embedded-hal` 1.0.

`M5ioe1::new` initializes the expander with `PYG11` parked, isolating the
IP2315 battery charge controller from the shared system bus. Typestate
transitions between parked and mounted modes prevent accidental communication
conflicts while managing external power sources.
