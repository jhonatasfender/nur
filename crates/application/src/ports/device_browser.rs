//! Porta para abrir o conteúdo de um dispositivo no gerenciador de arquivos.

use crate::errors::BrowseError;
use domain::DevicePath;

/// Abre o pendrive no gerenciador de arquivos do SO (montando se preciso).
#[async_trait::async_trait]
pub trait DeviceBrowser: Send + Sync {
    /// Abre o conteúdo do `device` no gerenciador nativo.
    ///
    /// # Errors
    /// Retorna [`BrowseError`] se não houver filesystem, ou se montar/abrir falhar.
    async fn open(&self, device: &DevicePath) -> Result<(), BrowseError>;
}
