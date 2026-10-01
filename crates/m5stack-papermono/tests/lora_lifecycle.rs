//! Register model for C153 control sequencing and cleanup. No physical device I/O.
use embedded_hal::i2c::{self, Operation};
use m5stack_papermono::lora::*;
use m5stack_papermono_lite::{addresses, m5ioe1, m5pm1};
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};

struct Model {
    expander: [u8; 256],
    pmic: [u8; 256],
    pointer: u8,
    writes: Vec<(u8, u8, u8)>,
    delays: Vec<u32>,
    events: Vec<&'static str>,
    fail_next: bool,
    fail_read: Option<u8>,
    sampled: Option<bool>,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            expander: [0; 256],
            pmic: [0; 256],
            pointer: 0,
            writes: vec![],
            delays: vec![],
            events: vec![],
            fail_next: false,
            fail_read: None,
            sampled: None,
        }
    }
}
#[derive(Clone)]
struct Bus(Rc<RefCell<Model>>);
impl i2c::ErrorType for Bus {
    type Error = i2c::ErrorKind;
}
impl i2c::I2c for Bus {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mut model = self.0.borrow_mut();
        if model.fail_next {
            model.fail_next = false;
            model.events.push("failed");
            return Err(i2c::ErrorKind::Other);
        }
        for op in operations {
            match op {
                Operation::Write(data) => {
                    model.pointer = data[0];
                    if data.len() == 2 {
                        let (reg, byte) = (data[0], data[1]);
                        model.writes.push((address, reg, byte));
                        if address == addresses::M5IOE1 {
                            model.expander[usize::from(reg)] = byte;
                            if reg == m5ioe1::GPIO_O_H {
                                model.events.push(if byte & 2 == 0 {
                                    "reset-low"
                                } else {
                                    "reset-high"
                                });
                            }
                            if reg == m5ioe1::GPIO_O_L {
                                model.events.push(if byte & 2 == 0 {
                                    "antenna-low"
                                } else {
                                    "antenna-high"
                                });
                            }
                        } else {
                            model.pmic[usize::from(reg)] = byte;
                            if reg == m5pm1::GPIO_OUT {
                                model.events.push(if byte & 4 == 0 {
                                    "rail-low"
                                } else {
                                    "rail-high"
                                });
                            }
                        }
                    }
                }
                Operation::Read(bytes) => {
                    let reg = model.pointer;
                    if model.fail_read == Some(reg) {
                        return Err(i2c::ErrorKind::Other);
                    }
                    let byte = if address == addresses::M5IOE1 {
                        if reg == m5ioe1::GPIO_I_L {
                            let high = model
                                .sampled
                                .unwrap_or(model.expander[usize::from(m5ioe1::GPIO_O_L)] & 2 != 0);
                            if high {
                                2
                            } else {
                                0
                            }
                        } else {
                            model.expander[usize::from(reg)]
                        }
                    } else {
                        model.pmic[usize::from(reg)]
                    };
                    bytes.fill(byte);
                }
            }
        }
        Ok(())
    }
}
struct Delay(Rc<RefCell<Model>>);
impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, ns: u32) {
        let mut model = self.0.borrow_mut();
        model.delays.push(ns / 1_000_000);
        model.events.push("delay");
    }
}
fn ready<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        _ => panic!("immediate model"),
    }
}
fn request(phase: CheckPhase) -> CheckRequest {
    CheckRequest {
        phase,
        interval: 1,
        attempt: 0,
        check: 1,
    }
}
#[test]
fn startup_and_shutdown_confirm_control_and_preserve_other_expander_pins() {
    let model = Rc::new(RefCell::new(Model::default()));
    model.borrow_mut().expander[usize::from(m5ioe1::GPIO_O_L)] = 0x80;
    let mut bus = Bus(model.clone());
    let mut hooks = RadioHooks::default();
    let mut c = RadioContext {
        i2c: &mut bus,
        delay: Delay(model.clone()),
        ioe_address: addresses::M5IOE1,
    };
    ready(hooks.startup(&mut c)).unwrap();
    assert!(ready(hooks.verify(&mut c, request(CheckPhase::Startup))).unwrap());
    assert_eq!(model.borrow().delays, [RAIL_SETTLE_MS, BOOT_SETTLE_MS]);
    let events = model.borrow().events.clone();
    let first = |event| events.iter().position(|s| *s == event).unwrap();
    assert!(first("reset-low") < first("antenna-high"));
    assert!(first("antenna-high") < first("rail-high"));
    assert!(first("rail-high") < first("reset-high"));
    assert_eq!(model.borrow().expander[usize::from(m5ioe1::GPIO_O_L)], 0x82);
    ready(hooks.shutdown(&mut c)).unwrap();
    assert!(ready(hooks.verify(&mut c, request(CheckPhase::Shutdown))).unwrap());
    assert_eq!(model.borrow().pmic[usize::from(m5pm1::GPIO_OUT)] & 4, 0);
}
#[test]
fn shutdown_attempts_antenna_and_rail_after_reset_control_fails() {
    let model = Rc::new(RefCell::new(Model::default()));
    let mut bus = Bus(model.clone());
    let mut hooks = RadioHooks::default();
    let mut c = RadioContext {
        i2c: &mut bus,
        delay: Delay(model.clone()),
        ioe_address: addresses::M5IOE1,
    };
    ready(hooks.startup(&mut c)).unwrap();
    model.borrow_mut().fail_next = true;
    assert!(ready(hooks.shutdown(&mut c)).is_err());
    assert!(ready(hooks.verify(&mut c, request(CheckPhase::Shutdown))).unwrap());
    let events = model.borrow().events.clone();
    let failed = events.iter().position(|s| *s == "failed").unwrap();
    assert!(events[failed..].contains(&"antenna-low"));
    assert!(events[failed..].contains(&"rail-low"));
}
#[test]
fn every_evidence_field_is_required_and_bus_failure_is_distinct() {
    let model = Rc::new(RefCell::new(Model::default()));
    let mut bus = Bus(model.clone());
    let mut hooks = RadioHooks::default();
    let mut c = RadioContext {
        i2c: &mut bus,
        delay: Delay(model.clone()),
        ioe_address: addresses::M5IOE1,
    };
    ready(hooks.startup(&mut c)).unwrap();
    for (reg, value) in [
        (m5ioe1::GPIO_M_L, 0),
        (m5ioe1::GPIO_DRV_L, 2),
        (m5ioe1::GPIO_O_L, 0),
    ] {
        let previous = model.borrow().expander[usize::from(reg)];
        model.borrow_mut().expander[usize::from(reg)] = value;
        assert!(!ready(hooks.verify(&mut c, request(CheckPhase::BeforeTx))).unwrap());
        model.borrow_mut().expander[usize::from(reg)] = previous;
    }
    model.borrow_mut().sampled = Some(false);
    assert!(!ready(hooks.verify(&mut c, request(CheckPhase::BeforeTx))).unwrap());
    model.borrow_mut().fail_read = Some(m5ioe1::GPIO_DRV_L);
    assert!(ready(hooks.verify(&mut c, request(CheckPhase::BeforeTx))).is_err());
}
