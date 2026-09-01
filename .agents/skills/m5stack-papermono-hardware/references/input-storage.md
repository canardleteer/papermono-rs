# Buttons, buzzer, SD, USB-C

## Keys

ESP32 GPIO2 = KEY1 (Button A), GPIO3 = KEY2 (Button B). GPIO3 is
a strapping pin. Power is the M5PM1 button, not a GPIO.
Physical edges:
[nyc-enclosure-edges](../resources/not-yet-confirmed.md#nyc-enclosure-edges).

## Buzzer

GPIO42 PWM (`BB_PWM`). Mux off JTAG `MTMS` (ESP32-S3 Table
2-4) before PWM. UserDemo LEDC: low-speed timer 3 / channel 7,
10-bit, 50% duty, 40–12000 Hz. Resonance / SPL:
[nyc-buzzer](../resources/not-yet-confirmed.md#nyc-buzzer).

## microSD

SDMMC: CMD 12, CLK 13, DAT0 11, DAT1 10, DAT2 9, DAT3 8.
Enable M5IOE1 `PYG14`. Detect `PYG1`, pulled up by the card
power domain; insert pulls low (official).

Official pin table and schematic name four data lines
(`DAT0` / `CD/DAT3` in the extract). UserDemo
`hal_tf_card.cpp` mounts **4-bit** (`slot_config.width = 4`)
after `PYG14` high and a 300 ms wait; detect insert = LOW.
FreeInk says native **1-bit** SDMMC. Wiring vs which width a
physical unit actually trains:
[nyc-sdmmc-width](../resources/not-yet-confirmed.md#nyc-sdmmc-width).
Detect polarity:
[nyc-tf-det](../resources/not-yet-confirmed.md#nyc-tf-det).

EPD is a different SPI controller. Overlapping SD and EPD
transactions is still a software mutex problem, not a shared
clock.

## USB-C

5 V sink. Schematic: USB-C CC1/CC2 **5.1 kΩ Rd** to GND; no
PD controller in the extract. **Lite run and download:**
`303a:1001` Espressif USB JTAG/serial debug unit
([flashing.md](flashing.md#usb-measured)). Vendor Arduino CDC
on boot is still intent. `C153`, `probe-rs`, extra CDC:
[nyc-usb-vid](../resources/not-yet-confirmed.md#nyc-usb-vid).
Not QinHeng `1a86:55d3`. Same VID:PID as many ESP32-S3
DevKits; pass `--port` if more than one is plugged in.

## NFC and LoRa (full SKU)

ST25R3916 `0x50` (sheet `50h`), IRQ GPIO6. Full schematic:
`I2C_EN=VDD; I2C mode`. UserDemo SKU probe and NFC app keep
the field off except while scanning
([user-demo.md](user-demo.md)). SX1262 GPIOs 38/40/39/41/21/5
(spec ≤16 MHz). Product table names that bus SPI1; UserDemo
uses `SPI3_HOST` at 8 MHz, RadioLib begin at 868.0 MHz (EU
demo vs product 868–923). Mux GPIO39–41 off JTAG before SPI.
Reset/ant via expander, enable M5PM1 G2.
Lite HTML **PinMap** omits RFID/LoRa; the Lite schematic still
draws those blocks (gallery page 05). Do not init; leftover
pads
[nyc-lite-nfc-pads](../resources/not-yet-confirmed.md#nyc-lite-nfc-pads)
/
[nyc-lite-lora-pads](../resources/not-yet-confirmed.md#nyc-lite-lora-pads).
ACK recipes:
[nyc-nfc-ack](../resources/not-yet-confirmed.md#nyc-nfc-ack),
[nyc-lora-ack](../resources/not-yet-confirmed.md#nyc-lora-ack).
