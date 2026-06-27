use super::*;

#[test]
fn humanizes_small_bytes() {
    assert_eq!(ByteSize::from_bytes(512).humanize(), "512 B");
}

#[test]
fn humanizes_gigabytes() {
    assert_eq!(ByteSize::from_bytes(32_000_000_000).humanize(), "32.0 GB");
}

#[test]
fn preserves_byte_count() {
    assert_eq!(ByteSize::from_bytes(1024).as_bytes(), 1024);
}

#[test]
fn avoids_rounding_to_1000_kb() {
    // 999_999 bytes = 999.999 KB, which rounds to 1000.0 KB — must show "1.0 MB".
    assert_eq!(ByteSize::from_bytes(999_999).humanize(), "1.0 MB");
}
