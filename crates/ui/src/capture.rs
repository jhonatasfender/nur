//! Captura de tela da janela do Nur (tecla F12 e modo automático).

use application::ports::ScreenshotWriter;
use std::path::PathBuf;
use std::sync::Arc;

/// Coordena capturas de tela da janela.
///
/// Dispara por **F12** (captura manual, numerada) ou pelo **modo automático**,
/// quando um destino é injetado pelo composition root — útil para validar a UI
/// de forma headless: renderiza alguns frames, grava o PNG e sinaliza para a
/// janela fechar. A gravação em arquivo é delegada ao [`ScreenshotWriter`], de
/// modo que a camada de apresentação não toca o sistema de arquivos.
pub(crate) struct Capturer {
    writer: Arc<dyn ScreenshotWriter>,
    auto: Option<PathBuf>,
    auto_requested: bool,
    frames: u32,
    counter: u32,
    // Há uma captura solicitada aguardando o evento `Event::Screenshot`.
    pending: bool,
}

impl Capturer {
    /// Cria o capturador com o gravador injetado e o destino automático opcional.
    #[must_use]
    pub(crate) fn new(writer: Arc<dyn ScreenshotWriter>, auto: Option<PathBuf>) -> Self {
        Self {
            writer,
            auto,
            auto_requested: false,
            frames: 0,
            counter: 0,
            pending: false,
        }
    }

    /// Define (ou limpa) o destino da captura automática.
    pub(crate) fn set_auto(&mut self, auto: Option<PathBuf>) {
        self.auto = auto;
    }

    /// Processa um frame: trata F12 / modo automático e salva screenshots prontos.
    ///
    /// Retorna `true` quando a captura automática concluiu (a janela deve fechar).
    pub(crate) fn process(&mut self, ctx: &egui::Context) -> bool {
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            Self::request(ctx);
            self.pending = true;
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
                self.pending = true;
            }
            if self.frames > MAX_FRAMES_AUTO {
                eprintln!(
                    "NUR_CAPTURE: screenshot não chegou após {MAX_FRAMES_AUTO} frames; abortando."
                );
                return true;
            }
        }
        // egui é reativo; enquanto há captura pendente, força os frames para o
        // evento `Event::Screenshot` chegar e ser salvo (vale para F12 também).
        if self.pending {
            ctx.request_repaint();
        }
        self.save_ready(ctx)
    }

    /// Solicita uma captura manualmente (ex.: botão na UI), sem depender do F12.
    pub(crate) fn capture_now(&mut self, ctx: &egui::Context) {
        Self::request(ctx);
        self.pending = true;
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
            // O evento chegou: a captura deixa de estar pendente.
            self.pending = false;
            let dest = self.next_destination();
            let [width, height] = image.size;
            match self
                .writer
                .write(image.as_raw(), width as u32, height as u32, &dest)
            {
                Ok(()) => auto_done = self.auto.is_some(),
                Err(e) => eprintln!("falha na captura: {e}"),
            }
        }
        auto_done
    }

    // Destino do PNG: o caminho automático injetado ou um nome numerado relativo
    // ao diretório atual do processo (resolvido pelo gravador, na infraestrutura).
    fn next_destination(&mut self) -> PathBuf {
        if let Some(path) = &self.auto {
            return path.clone();
        }
        self.counter += 1;
        PathBuf::from(format!("nur-screenshot-{:03}.png", self.counter))
    }
}

#[cfg(test)]
mod tests;
