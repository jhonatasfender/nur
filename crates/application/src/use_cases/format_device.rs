//! Caso de uso: formatar um dispositivo.

use crate::errors::FormatError;
use crate::ports::{DeviceFormatter, FormatOptions, ProgressSink, WritePhase, WriteProgress};
use domain::DevicePath;
use std::sync::Arc;

/// Orquestra a formatação do dispositivo.
pub struct FormatDevice {
    formatter: Arc<dyn DeviceFormatter>,
}

impl FormatDevice {
    /// Cria o caso de uso com a porta injetada.
    #[must_use]
    pub fn new(formatter: Arc<dyn DeviceFormatter>) -> Self {
        Self { formatter }
    }

    /// Reporta o início e formata o dispositivo.
    ///
    /// # Errors
    /// Retorna [`FormatError`] se a formatação falhar.
    pub async fn execute(
        &self,
        device: DevicePath,
        options: FormatOptions,
        sink: Arc<dyn ProgressSink>,
    ) -> Result<(), FormatError> {
        sink.report(WriteProgress::new(WritePhase::Preparing, 0, 0));
        self.formatter.format(&device, &options).await
    }
}

#[cfg(test)]
mod tests;
