use super::IsoFileInspector;
use domain::IsoKind;

fn mbr_with_partition() -> [u8; 512] {
    let mut b = [0u8; 512];
    b[510] = 0x55;
    b[511] = 0xAA;
    // 1ª entrada de partição não-vazia (offset 0x1BE), tipo 0x83.
    b[0x1BE + 4] = 0x83;
    b
}

#[test]
fn isohybrid_when_signature_and_partition_present() {
    assert_eq!(
        IsoFileInspector::classify_bytes(&mbr_with_partition(), true),
        IsoKind::Isohybrid
    );
}

#[test]
fn unsupported_when_no_signature() {
    let mut b = mbr_with_partition();
    b[510] = 0x00;
    assert_eq!(
        IsoFileInspector::classify_bytes(&b, true),
        IsoKind::Unsupported
    );
}

#[test]
fn unsupported_when_no_partition() {
    let mut b = [0u8; 512];
    b[510] = 0x55;
    b[511] = 0xAA;
    assert_eq!(
        IsoFileInspector::classify_bytes(&b, true),
        IsoKind::Unsupported
    );
}
