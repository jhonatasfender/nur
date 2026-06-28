use super::{CancelFlag, WritePhase, WriteProgress};

#[test]
fn cancel_flag_starts_unset_and_latches() {
    let flag = CancelFlag::new();
    assert!(!flag.is_cancelled());
    flag.cancel();
    assert!(flag.is_cancelled());
}

#[test]
fn cancel_flag_clone_shares_state() {
    let flag = CancelFlag::new();
    let clone = flag.clone();
    flag.cancel();
    assert!(clone.is_cancelled());
}

#[test]
fn write_progress_exposes_fields() {
    let p = WriteProgress::new(WritePhase::Writing, 10, 100);
    assert_eq!(p.phase(), WritePhase::Writing);
    assert_eq!(p.done(), 10);
    assert_eq!(p.total(), 100);
}
