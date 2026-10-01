//! Serial evidence round trips and worst-case fixed buffer sizing.
use papermono_log::parse::{records, LineKind};
use papermono_log::*;
#[test]
fn unknown_evidence_is_distinct_and_warnings_are_typed() {
    let mut buffer = [0; LORA_CONTROL_CAPACITY];
    let line = format_lora_control(
        &LoraControlSample {
            phase: LoraPhase::BeforeTx,
            interval: 1,
            attempts: 1,
            checks: 2,
            expected_high: true,
            output: Some(true),
            push_pull: Some(true),
            latch: Some(true),
            level: None,
            failure: LoraFailure::Bus,
        },
        &mut buffer,
    )
    .unwrap();
    assert_eq!(line,"simple-debug: lora_control phase=tx n=1 attempts=1 checks=2 expected=1 mode=1 drive=1 latch=1 observed=unknown reason=bus warning=1");
    assert_eq!(records(line).next().unwrap().kind, LineKind::LoraControl);
    let mut tiny = [0; 5];
    assert!(format_lora_control(
        &LoraControlSample {
            phase: LoraPhase::Shutdown,
            interval: 40,
            attempts: u64::MAX,
            checks: u64::MAX,
            expected_high: false,
            output: None,
            push_pull: None,
            latch: None,
            level: None,
            failure: LoraFailure::Mismatch
        },
        &mut tiny
    )
    .is_err());
    assert!(format_lora_control(
        &LoraControlSample {
            phase: LoraPhase::Shutdown,
            interval: u32::MAX,
            attempts: u64::MAX,
            checks: u64::MAX,
            expected_high: false,
            output: None,
            push_pull: None,
            latch: None,
            level: None,
            failure: LoraFailure::Mismatch
        },
        &mut buffer
    )
    .is_ok());
}
#[test]
fn summary_contains_counters_and_cleanup_failure() {
    let mut buffer = [0; LORA_SESSION_CAPACITY];
    let line = format_lora_session(
        &LoraSessionSample {
            interval: 1,
            attempts: 20,
            checks: 22,
            failures: 1,
            cleanup_ok: false,
        },
        &mut buffer,
    )
    .unwrap();
    assert_eq!(
        line,
        "simple-debug: lora_session n=1 attempts=20 checks=22 failures=1 cleanup=failed"
    );
    assert_eq!(records(line).next().unwrap().kind, LineKind::LoraSession);
    assert!(format_lora_session(
        &LoraSessionSample {
            interval: u32::MAX,
            attempts: u64::MAX,
            checks: u64::MAX,
            failures: u64::MAX,
            cleanup_ok: true
        },
        &mut buffer
    )
    .is_ok());
}
