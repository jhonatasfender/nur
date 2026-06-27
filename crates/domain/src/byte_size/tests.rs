use super::*;

#[test]
fn humaniza_bytes_pequenos() {
    assert_eq!(ByteSize::from_bytes(512).humanize(), "512 B");
}

#[test]
fn humaniza_gigabytes() {
    assert_eq!(ByteSize::from_bytes(32_000_000_000).humanize(), "32.0 GB");
}

#[test]
fn preserva_contagem_de_bytes() {
    assert_eq!(ByteSize::from_bytes(1024).as_bytes(), 1024);
}
