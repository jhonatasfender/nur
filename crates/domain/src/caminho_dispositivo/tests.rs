use super::*;

#[test]
fn expoe_o_caminho() {
    let c = CaminhoDispositivo::new("/dev/sdb".to_owned());
    assert_eq!(c.as_str(), "/dev/sdb");
}
