//! Captura de tela da janela do Nur (tecla F12 e modo automático via env).

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Coordena capturas de tela da janela.
///
/// Dispara por **F12** (captura manual, numerada) ou pelo **modo automático**,
/// ativado pela variável de ambiente `NUR_CAPTURE=<arquivo.png>` — útil para
/// validar a UI de forma headless: renderiza alguns frames, salva o PNG e
/// sinaliza para a janela fechar.
pub struct Capturer {
    auto: Option<PathBuf>,
    auto_requested: bool,
    frames: u32,
    counter: u32,
    last_msg: Option<String>,
}

impl Capturer {
    /// Cria o capturador, lendo `NUR_CAPTURE` para o modo automático.
    #[must_use]
    pub fn new() -> Self {
        let auto = std::env::var_os("NUR_CAPTURE").map(PathBuf::from);
        Self {
            auto,
            auto_requested: false,
            frames: 0,
            counter: 0,
            last_msg: None,
        }
    }

    /// Indica se a captura automática está configurada.
    #[must_use]
    pub fn auto_enabled(&self) -> bool {
        self.auto.is_some()
    }

    /// Mensagem de status da última captura, se houver.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.last_msg.as_deref()
    }

    /// Processa um frame: trata F12 / modo automático e salva screenshots prontos.
    ///
    /// Retorna `true` quando a captura automática concluiu (a janela deve fechar).
    pub fn process(&mut self, ctx: &egui::Context) -> bool {
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            Self::request(ctx);
        }
        if self.auto.is_some() {
            // Failsafe: se o screenshot não chegar (ex.: sem framebuffer),
            // aborta em vez de girar para sempre.
            const MAX_FRAMES_AUTO: u32 = 600;
            self.frames += 1;
            // Espera a UI estabilizar antes de capturar no modo automático.
            if self.frames >= 3 && !self.auto_requested {
                Self::request(ctx);
                self.auto_requested = true;
            }
            if self.frames > MAX_FRAMES_AUTO {
                eprintln!(
                    "NUR_CAPTURE: screenshot não chegou após {MAX_FRAMES_AUTO} frames; abortando."
                );
                return true;
            }
            // egui é reativo; força frames para o screenshot chegar.
            ctx.request_repaint();
        }
        self.save_ready(ctx)
    }

    fn request(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
    }

    fn save_ready(&mut self, ctx: &egui::Context) -> bool {
        let images: Vec<Arc<egui::ColorImage>> = ctx.input(|i| {
            i.raw
                .events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        let mut auto_done = false;
        for image in images {
            let dest = self.next_destination();
            match Self::save_png(&image, &dest) {
                Ok(()) => {
                    self.last_msg = Some(format!("captura salva em {}", dest.display()));
                    auto_done = self.auto.is_some();
                }
                Err(e) => self.last_msg = Some(format!("falha na captura: {e}")),
            }
        }
        auto_done
    }

    fn next_destination(&mut self) -> PathBuf {
        if let Some(path) = &self.auto {
            return path.clone();
        }
        self.counter += 1;
        PathBuf::from(format!("nur-screenshot-{:03}.png", self.counter))
    }

    fn save_png(image: &egui::ColorImage, dest: &Path) -> Result<(), image::ImageError> {
        let [width, height] = image.size;
        image::save_buffer(
            dest,
            image.as_raw(),
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )
    }
}

impl Default for Capturer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
