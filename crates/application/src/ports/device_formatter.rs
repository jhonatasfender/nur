//! Porta de formatação de um dispositivo.

use crate::errors::FormatError;
use crate::ports::FormatOptions;
use domain::DevicePath;

/// Formata o dispositivo (tabela de partição + 1 partição + filesystem).
#[async_trait::async_trait]
pub trait DeviceFormatter: Send + Sync {
    /// Formata o `device` conforme as `options`.
    ///
    /// # Errors
    /// Retorna [`FormatError`] em falha de autorização, ferramenta ausente ou backend.
    async fn format(
        &self,
        device: &DevicePath,
        options: &FormatOptions,
    ) -> Result<(), FormatError>;
}
