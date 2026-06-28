//! Adapter stub do DiskService (dados canônicos para preview/desenvolvimento).

use application::errors::DiskError;
use application::ports::DiskService;
use domain::{ByteSize, Device, DevicePath};

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

#[async_trait::async_trait]
impl DiskService for DiskServiceStub {
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        Ok(vec![
            Device::new(
                DevicePath::new("/dev/sdb".to_owned()),
                "SanDisk Ultra".to_owned(),
                ByteSize::from_bytes(32_000_000_000),
                true,
            ),
            Device::new(
                DevicePath::new("/dev/sdc".to_owned()),
                "Kingston DataTraveler".to_owned(),
                ByteSize::from_bytes(16_000_000_000),
                true,
            ),
        ])
    }
}

#[cfg(test)]
mod tests;
