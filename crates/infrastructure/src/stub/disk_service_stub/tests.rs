use super::*;

#[test]
fn returns_two_canonical_devices() {
    let stub = DiskServiceStub::new();
    let devices = stub.list_devices().unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].path().as_str(), "/dev/sdb");
}
