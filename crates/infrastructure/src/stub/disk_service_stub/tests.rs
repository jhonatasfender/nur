use super::*;

#[test]
fn devolve_dois_dispositivos_canonicos() {
    let stub = DiskServiceStub::new();
    let dispositivos = stub.listar_dispositivos().unwrap();
    assert_eq!(dispositivos.len(), 2);
    assert_eq!(dispositivos[0].caminho().as_str(), "/dev/sdb");
}
