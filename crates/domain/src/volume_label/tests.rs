use super::*;

#[test]
fn accepts_valid_label() {
    let r = VolumeLabel::parse("BOOTUSB").unwrap();
    assert_eq!(r.as_str(), "BOOTUSB");
}

#[test]
fn rejects_empty() {
    assert!(VolumeLabel::parse("").is_err());
}

#[test]
fn rejects_above_11_chars() {
    assert!(VolumeLabel::parse("ABCDEFGHIJKL").is_err());
}
