use super::*;
use crate::ports::WriteState;

#[test]
fn ui_state_defaults_are_idle_and_none() {
    struct Minimal;
    impl UiState for Minimal {
        fn device_list(&self) -> DeviceListState {
            DeviceListState::Loading
        }
    }
    let s = Minimal;
    assert_eq!(s.write_state(), WriteState::Idle);
    assert!(s.selected_iso().is_none());
}

#[test]
fn ready_carries_devices() {
    let state =
        DeviceListState::Ready(vec![DeviceView::new("/dev/sdb".to_owned(), "x".to_owned())]);
    match state {
        DeviceListState::Ready(v) => assert_eq!(v.len(), 1),
        _ => panic!("esperava Ready"),
    }
}
