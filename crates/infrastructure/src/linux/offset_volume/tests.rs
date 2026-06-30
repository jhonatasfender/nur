use super::OffsetVolume;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

fn base() -> Cursor<Vec<u8>> {
    Cursor::new(vec![0u8; 100])
}

#[test]
fn writes_and_reads_within_window() {
    let mut v = OffsetVolume::new(base(), 10, 20);
    v.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(v.write(&[1, 2, 3]).unwrap(), 3);
    v.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = [0u8; 3];
    v.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [1, 2, 3]);
}

#[test]
fn write_is_clamped_to_window_len() {
    let mut v = OffsetVolume::new(base(), 10, 4);
    v.seek(SeekFrom::Start(2)).unwrap();
    let n = v.write(&[9, 9, 9, 9]).unwrap();
    assert_eq!(n, 2);
}

#[test]
fn maps_offset_into_base() {
    let mut v = OffsetVolume::new(base(), 10, 20);
    v.seek(SeekFrom::Start(5)).unwrap();
    v.write_all(&[7]).unwrap();
    let inner = v.into_inner();
    assert_eq!(inner.into_inner()[15], 7);
}
