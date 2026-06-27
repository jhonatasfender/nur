use super::*;

#[test]
fn builds_readable_description() {
    let d = Device::new(
        DevicePath::new("/dev/sdb".to_owned()),
        "SanDisk Ultra".to_owned(),
        ByteSize::from_bytes(32_000_000_000),
        true,
    );
    assert_eq!(d.description(), "SanDisk Ultra — 32.0 GB (/dev/sdb)");
    assert!(d.removable());
}
