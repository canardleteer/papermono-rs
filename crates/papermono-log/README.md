# papermono-log

Host-tested USB-Serial/JTAG log contract for the PaperMono
firmware images (`firmware/simple-debug` and
`firmware/embassy-debug`). The wire prefix stays
`simple-debug:`.

This crate owns the line format. The Xtensa images map pins and
print. No MAC, BSSID, IRK, USB serial, or flash identifiers.

Kinds (CRLF on the wire):

```text
simple-debug: hello t=0 image=simple-debug sku=C153-Lite chip=esp32s3 cpu_mhz=80 xtal_mhz=40 reset=chip_power_on
simple-debug: git=<hash> dirty=<0|1>
simple-debug: gpio boot=1 pmic_irq=0 tp=0 ioe=1 busy=0
simple-debug: leftover lora_irq=0 nfc_irq=0 sx_busy=0
simple-debug: hb t=12 btn_a=1 btn_b=0
simple-debug: edge t_ms=1250 btn_a=1->0
simple-debug: i2c pm1=1 ioe=1 ioe_addr=4f rtc=1 rtc_flag=00 imu=1 imu_id=24 tp=1 nfc=0 chg=0 tf=1 ack=32,38,4f,68,6e nak=50,6f,75
simple-debug: touch int=0 n=0
simple-debug: mic rms=12 peak=40
simple-debug: panel mode=otp_orient w=800 h=480 busy_rose=1
simple-debug: scene=splash
simple-debug: lamp=1024
simple-debug: wifi n=12
simple-debug: ble n=4
simple-debug: charge vbat=3921 vin=5080 src=05 chg_en=1 ip=1 then=0
```

`hello image=` is `simple-debug` or `embassy-debug`. Split glued
CDC lines on `simple-debug:` as well as newline.

`hello` / `git` / `gpio` repeat every 10 s so a late CDC attach
still sees identity. Heartbeat is 1 Hz. Edges are 50 ms polls.
