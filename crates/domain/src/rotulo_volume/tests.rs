use super::*;

#[test]
fn aceita_rotulo_valido() {
    let r = RotuloVolume::parse("BOOTUSB").unwrap();
    assert_eq!(r.as_str(), "BOOTUSB");
}

#[test]
fn rejeita_vazio() {
    assert!(RotuloVolume::parse("").is_err());
}

#[test]
fn rejeita_acima_de_11_chars() {
    assert!(RotuloVolume::parse("ABCDEFGHIJKL").is_err());
}
