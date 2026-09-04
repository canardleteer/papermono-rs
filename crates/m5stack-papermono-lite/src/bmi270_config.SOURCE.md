# BMI270 standard config blob

Rust module: `bmi270_config.rs` (8192 bytes).

Provenance: Bosch Sensortec
[BMI270_SensorAPI](https://github.com/boschsensortec/BMI270_SensorAPI)
`bmi270_config_file` in `bmi270.c` (BSD-3-Clause). Required after
POR / soft-reset before raw `DATA_8`…`DATA_13` samples are valid
on CHIP_ID `0x24`. Upload uses `INIT_ADDR_0`/`INIT_ADDR_1` plus
`INIT_DATA` per the Sensor API `upload_file` sequence.

The smaller `bmi270_maximum_fifo` blob is a **different product
variant** and must not be used on PaperMono’s BMI270.
