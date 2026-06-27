//! Dispositivo de bloco detectado (pendrive) como agregado de domínio.

use crate::{ByteSize, DevicePath};

/// Um dispositivo de armazenamento detectado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    path: DevicePath,
    model: String,
    size: ByteSize,
    removable: bool,
}

impl Device {
    /// Cria um dispositivo a partir dos dados do adapter de SO.
    #[must_use]
    pub fn new(path: DevicePath, model: String, size: ByteSize, removable: bool) -> Self {
        Self {
            path,
            model,
            size,
            removable,
        }
    }

    /// Caminho do dispositivo.
    #[must_use]
    pub fn path(&self) -> &DevicePath {
        &self.path
    }

    /// Modelo do dispositivo.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Tamanho do dispositivo.
    #[must_use]
    pub fn size(&self) -> ByteSize {
        self.size
    }

    /// Indica se o dispositivo é removível.
    #[must_use]
    pub fn removable(&self) -> bool {
        self.removable
    }

    /// Descrição legível para a UI (modelo — tamanho (caminho)).
    #[must_use]
    pub fn description(&self) -> String {
        format!(
            "{} — {} ({})",
            self.model,
            self.size.humanize(),
            self.path.as_str()
        )
    }
}

#[cfg(test)]
mod tests;
