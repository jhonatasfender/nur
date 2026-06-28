//! Seletor de arquivo ISO usando o diálogo nativo (rfd).

use std::path::PathBuf;

/// Abre o diálogo nativo para escolher uma imagem.
pub struct RfdIsoPicker;

impl RfdIsoPicker {
    /// Cria o seletor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Abre o diálogo e devolve o caminho escolhido, se houver.
    pub async fn pick(&self) -> Option<PathBuf> {
        rfd::AsyncFileDialog::new()
            .add_filter("Imagens", &["iso", "img"])
            .pick_file()
            .await
            .map(|h| h.path().to_path_buf())
    }
}

impl Default for RfdIsoPicker {
    fn default() -> Self {
        Self::new()
    }
}
