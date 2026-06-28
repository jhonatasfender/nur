//! Caso de uso: criar um pendrive bootável a partir de uma ISO.

use crate::errors::WriteError;
use crate::ports::{BootableWriter, CancelFlag, IsoInspector, ProgressSink, WriteRequest};
use domain::IsoKind;
use std::sync::Arc;

/// Orquestra a classificação da ISO e a gravação no dispositivo.
pub struct CreateBootable {
    inspector: Arc<dyn IsoInspector>,
    writer: Arc<dyn BootableWriter>,
}

impl CreateBootable {
    /// Cria o caso de uso com as portas injetadas.
    #[must_use]
    pub fn new(inspector: Arc<dyn IsoInspector>, writer: Arc<dyn BootableWriter>) -> Self {
        Self { inspector, writer }
    }

    /// Classifica a ISO e, se gravável por raw, grava e verifica.
    ///
    /// # Errors
    /// Retorna [`WriteError`] se a ISO não for suportada ou se a gravação falhar.
    pub async fn execute(
        &self,
        request: WriteRequest,
        sink: Arc<dyn ProgressSink>,
        cancel: CancelFlag,
    ) -> Result<(), WriteError> {
        let kind = self
            .inspector
            .classify(request.iso_path())
            .await
            .map_err(|e| WriteError::Io(e.to_string()))?;
        if kind == IsoKind::Unsupported {
            return Err(WriteError::Io(
                "ISO não suportada para gravação raw (provavelmente Windows); \
                 modo de extração ainda não disponível"
                    .to_owned(),
            ));
        }
        self.writer.write(&request, sink, &cancel).await
    }
}

#[cfg(test)]
mod tests;
