//! Adapter de disco para Linux lendo o sysfs (`/sys/block`). Rápido, sem D-Bus.
//!
//! O udisks2 (zbus) ficou pesado para enumeração (muitas chamadas D-Bus, ~1s
//! cada). O sysfs expõe os mesmos dados em leituras de arquivo locais — usado
//! aqui para listar pendrives. O udisks2 fica para a gravação (polkit).

use application::errors::DiskError;
use application::ports::DiskService;
use domain::{ByteSize, Device, DevicePath};
use std::path::Path;

/// Lista pendrives (dispositivos de bloco USB) lendo o `/sys/block`.
pub struct SysfsDiskService;

impl SysfsDiskService {
    /// Cria o adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Monta um Device a partir dos campos do sysfs; `Some` apenas se for USB
    // (o caminho canônico em `/sys` contém `/usb` quando conectado por USB).
    fn build_device(
        name: &str,
        canonical: &str,
        size_sectors: u64,
        removable: bool,
        model: &str,
    ) -> Option<Device> {
        if !canonical.contains("/usb") {
            return None;
        }
        Some(Device::new(
            DevicePath::new(format!("/dev/{name}")),
            model.trim().to_owned(),
            ByteSize::from_bytes(size_sectors.saturating_mul(512)),
            removable,
        ))
    }

    // Lê os campos do sysfs para um dispositivo de bloco.
    fn read_device(name: &str) -> Option<Device> {
        let base = Path::new("/sys/block").join(name);
        let canonical = std::fs::canonicalize(&base).ok()?;
        let size: u64 = std::fs::read_to_string(base.join("size"))
            .ok()?
            .trim()
            .parse()
            .ok()?;
        let removable =
            std::fs::read_to_string(base.join("removable")).is_ok_and(|s| s.trim() == "1");
        let model = std::fs::read_to_string(base.join("device/model")).unwrap_or_default();
        Self::build_device(name, &canonical.to_string_lossy(), size, removable, &model)
    }

    // Varre `/sys/block` e coleta os pendrives.
    fn collect() -> Result<Vec<Device>, DiskError> {
        let entries =
            std::fs::read_dir("/sys/block").map_err(|e| DiskError::Unavailable(e.to_string()))?;
        let mut devices = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(device) = Self::read_device(name) {
                    devices.push(device);
                }
            }
        }
        Ok(devices)
    }
}

impl Default for SysfsDiskService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DiskService for SysfsDiskService {
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        tokio::task::spawn_blocking(Self::collect)
            .await
            .map_err(|e| DiskError::Backend(e.to_string()))?
    }
}

#[cfg(test)]
mod tests;
