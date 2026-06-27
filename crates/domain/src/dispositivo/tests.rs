use super::*;

#[test]
fn monta_descricao_legivel() {
    let d = Dispositivo::new(
        CaminhoDispositivo::new("/dev/sdb".to_owned()),
        "SanDisk Ultra".to_owned(),
        ByteSize::from_bytes(32_000_000_000),
        true,
    );
    assert_eq!(d.descricao(), "SanDisk Ultra — 32.0 GB (/dev/sdb)");
    assert!(d.removivel());
}
