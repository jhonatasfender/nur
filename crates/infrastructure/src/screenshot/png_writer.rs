//! Adapter que grava capturas de tela em PNG usando o crate `image`.

use application::errors::ScreenshotError;
use application::ports::ScreenshotWriter;
use std::path::Path;

/// Gravador de capturas que persiste os pixels RGBA como PNG.
pub struct PngScreenshotWriter;

impl PngScreenshotWriter {
    /// Cria o gravador PNG.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PngScreenshotWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenshotWriter for PngScreenshotWriter {
    fn write(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
        dest: &Path,
    ) -> Result<(), ScreenshotError> {
        image::save_buffer(dest, rgba, width, height, image::ExtendedColorType::Rgba8)
            .map_err(|e| ScreenshotError::Write(e.to_string()))
    }
}

#[cfg(test)]
mod tests;
