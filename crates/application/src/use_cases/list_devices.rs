//! Caso de uso: listar dispositivos disponíveis para exibição.

use crate::errors::DiskError;
use crate::ports::{DeviceView, DiskService};
use std::sync::Arc;

/// Lista dispositivos e os projeta para a UI.
pub struct ListDevices {
    service: Arc<dyn DiskService>,
}

impl ListDevices {
    /// Cria o caso de uso com a porta de disco injetada.
    #[must_use]
    pub fn new(service: Arc<dyn DiskService>) -> Self {
        Self { service }
    }

    /// Executa a listagem e mapeia para [`DeviceView`].
    ///
    /// # Errors
    /// Propaga [`DiskError`] do backend.
    pub async fn execute(&self) -> Result<Vec<DeviceView>, DiskError> {
        let devices = self.service.list_devices().await?;
        Ok(devices
            .into_iter()
            .map(|d| DeviceView::new(d.path().as_str().to_owned(), d.description()))
            .collect())
    }
}

#[cfg(test)]
mod tests;
