# m5pm1

Register definitions, PWM helpers, and I2C communication for the M5Stack
M5PM1 Power Management IC.

Datasheet: [M5Stack M5PM1 Datasheet V 1.9](https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1207/M5PM1_Datasheet_EN.pdf).
Additional device details appear on the
[M5Stack Documentation](https://docs.m5stack.com) portal.

All operations run without dynamic memory allocations under `#![no_std]`
using `embedded-hal` 1.0 traits.

On PaperMono hardware, PWM0 drives frontlight brightness as a multiplexed
channel over GPIO3, whereas PWM1 runs on an independent timer. The driver
supports analog-to-digital conversions, battery status reads, and software
power-down through an explicit method.
