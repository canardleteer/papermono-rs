//! Parked GPIO42 LEDC chirp.
//!
//! The first image that took GPIO42 (JTAG `MTMS`) for UserDemo
//! LEDC timer 3 / channel 7 coincided with wedged card flips
//! (`gpio busy=1`, Ferris stuck) and a lamp whose `lamp=` duty
//! moved on CDC while the LED did not. Do not mux this pad
//! again until `nyc-buzzer` is isolated. USB-Serial/JTAG is
//! the USB PHY, but Table 2-4 still names GPIO42 `MTMS`.

/// No-op. LEDC and GPIO42 stay untouched.
#[allow(dead_code)]
pub fn ask() {}
