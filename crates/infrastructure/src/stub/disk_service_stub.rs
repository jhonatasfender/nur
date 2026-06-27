//! Adapter stub do DiskService (dados canônicos para preview/desenvolvimento).

use application::erros::ErroDisco;
use application::ports::DiskService;
use domain::{ByteSize, CaminhoDispositivo, Dispositivo};

/// Implementação stub que devolve dispositivos fixos (sem tocar o SO).
pub struct DiskServiceStub;

impl DiskServiceStub {
    /// Cria o stub.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiskServiceStub {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskService for DiskServiceStub {
    fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco> {
        Ok(vec![
            Dispositivo::new(
                CaminhoDispositivo::new("/dev/sdb".to_owned()),
                "SanDisk Ultra".to_owned(),
                ByteSize::from_bytes(32_000_000_000),
                true,
            ),
            Dispositivo::new(
                CaminhoDispositivo::new("/dev/sdc".to_owned()),
                "Kingston DataTraveler".to_owned(),
                ByteSize::from_bytes(16_000_000_000),
                true,
            ),
        ])
    }
}

#[cfg(test)]
mod tests;
