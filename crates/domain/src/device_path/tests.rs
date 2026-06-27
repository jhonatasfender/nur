use super::*;

#[test]
fn exposes_path() {
    let c = DevicePath::new("/dev/sdb".to_owned());
    assert_eq!(c.as_str(), "/dev/sdb");
}
