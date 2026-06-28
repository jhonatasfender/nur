//! Porta de inspeção de ISO (classifica o tipo da imagem).

use crate::errors::IsoError;
use domain::IsoKind;
use std::path::Path;

/// Inspeciona uma imagem ISO e a classifica.
#[async_trait::async_trait]
pub trait IsoInspector: Send + Sync {
    /// Classifica a ISO no caminho dado.
    ///
    /// # Errors
    /// Retorna [`IsoError`] se a leitura falhar.
    async fn classify(&self, iso: &Path) -> Result<IsoKind, IsoError>;
}
