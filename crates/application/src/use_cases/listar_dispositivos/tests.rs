use super::*;
use domain::{ByteSize, CaminhoDispositivo, Dispositivo};

struct DiskServiceFake;
impl DiskService for DiskServiceFake {
    fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco> {
        Ok(vec![Dispositivo::new(
            CaminhoDispositivo::new("/dev/sdb".to_owned()),
            "SanDisk Ultra".to_owned(),
            ByteSize::from_bytes(32_000_000_000),
            true,
        )])
    }
}

#[test]
fn mapeia_dispositivos_para_views() {
    let uc = ListarDispositivos::new(Arc::new(DiskServiceFake));
    let views = uc.executar().unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].caminho, "/dev/sdb");
    assert!(views[0].descricao.contains("SanDisk Ultra"));
}
