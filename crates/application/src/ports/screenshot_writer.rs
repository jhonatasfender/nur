//! Porta de gravação de capturas de tela: a UI entrega pixels, a infra grava.

use crate::errors::ScreenshotError;
use std::path::Path;

/// Persiste uma captura de tela a partir de pixels RGBA.
///
/// A camada de apresentação produz os bytes (`width` × `height`, RGBA8) e
/// delega a gravação em arquivo para um adapter da infraestrutura.
pub trait ScreenshotWriter: Send + Sync {
    /// Grava os pixels RGBA no destino indicado.
    ///
    /// # Errors
    /// Retorna [`ScreenshotError`] se a gravação falhar.
    fn write(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
        dest: &Path,
    ) -> Result<(), ScreenshotError>;
}
