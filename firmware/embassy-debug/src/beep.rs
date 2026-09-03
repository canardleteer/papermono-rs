//! Passive buzzer driver isolation and GPIO42 pin parking documentation.
//!
//! # Hardware Topology & Electrical Constraints
//! The board connects a passive magnetic buzzer to `GPIO42`:
//!
//! - **Pin Multiplexing Conflict**: `GPIO42` is shared with JTAG `MTMS` (ESP32-S3
//!   datasheet Table 2-4). In initial bring-up, attaching an active LEDC timer
//!   waveform to `GPIO42` interfered with internal system bus operations, leading
//!   to frozen card transitions and stuck EPD refresh cycles (`BUSY=1`).
//! - **Safe State**: To guarantee firmware stability, `GPIO42` is kept dormant
//!   (parked) as a high-impedance pin without attaching LEDC timers or PWM generators
//!   until dedicated electrical isolation is verified.

/// Intentionally empty placeholder function; keeps `GPIO42` untouched.
#[allow(dead_code)]
pub fn ask() {}
