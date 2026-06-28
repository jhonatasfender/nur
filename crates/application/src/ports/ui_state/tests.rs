use super::*;

#[test]
fn ready_carries_devices() {
    let state =
        DeviceListState::Ready(vec![DeviceView::new("/dev/sdb".to_owned(), "x".to_owned())]);
    match state {
        DeviceListState::Ready(v) => assert_eq!(v.len(), 1),
        _ => panic!("esperava Ready"),
    }
}
