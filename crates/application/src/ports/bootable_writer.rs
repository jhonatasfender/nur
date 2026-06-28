//! Porta de gravação de ISO bootável num dispositivo.

use crate::errors::WriteError;
use crate::ports::{CancelFlag, ProgressSink, WriteRequest};
use std::sync::Arc;

/// Grava a imagem no dispositivo, reportando progresso e respeitando cancelamento.
#[async_trait::async_trait]
pub trait BootableWriter: Send + Sync {
    /// Grava e verifica a imagem do `request` no dispositivo de destino.
    ///
    /// # Errors
    /// Retorna [`WriteError`] em falha de autorização, IO, verificação ou cancelamento.
    async fn write(
        &self,
        request: &WriteRequest,
        sink: Arc<dyn ProgressSink>,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError>;
}
