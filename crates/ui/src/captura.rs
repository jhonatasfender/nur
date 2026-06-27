//! Captura de tela da janela do Nur (tecla F12 e modo automático via env).

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Coordena capturas de tela da janela.
///
/// Dispara por **F12** (captura manual, numerada) ou pelo **modo automático**,
/// ativado pela variável de ambiente `NUR_CAPTURE=<arquivo.png>` — útil para
/// validar a UI de forma headless: renderiza alguns frames, salva o PNG e
/// sinaliza para a janela fechar.
pub struct Capturador {
    auto: Option<PathBuf>,
    auto_solicitado: bool,
    frames: u32,
    contador: u32,
    ultima_msg: Option<String>,
}

impl Capturador {
    /// Cria o capturador, lendo `NUR_CAPTURE` para o modo automático.
    #[must_use]
    pub fn new() -> Self {
        let auto = std::env::var_os("NUR_CAPTURE").map(PathBuf::from);
        Self {
            auto,
            auto_solicitado: false,
            frames: 0,
            contador: 0,
            ultima_msg: None,
        }
    }

    /// Indica se a captura automática está configurada.
    #[must_use]
    pub fn auto_ativo(&self) -> bool {
        self.auto.is_some()
    }

    /// Mensagem de status da última captura, se houver.
    #[must_use]
    pub fn mensagem(&self) -> Option<&str> {
        self.ultima_msg.as_deref()
    }

    /// Processa um frame: trata F12 / modo automático e salva screenshots prontos.
    ///
    /// Retorna `true` quando a captura automática concluiu (a janela deve fechar).
    pub fn processar(&mut self, ctx: &egui::Context) -> bool {
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            Self::solicitar(ctx);
        }
        if self.auto.is_some() {
            self.frames += 1;
            // Espera a UI estabilizar antes de capturar no modo automático.
            if self.frames >= 3 && !self.auto_solicitado {
                Self::solicitar(ctx);
                self.auto_solicitado = true;
            }
            // egui é reativo; força frames para o screenshot chegar.
            ctx.request_repaint();
        }
        self.salvar_prontos(ctx)
    }

    fn solicitar(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
    }

    fn salvar_prontos(&mut self, ctx: &egui::Context) -> bool {
        let imagens: Vec<Arc<egui::ColorImage>> = ctx.input(|i| {
            i.raw
                .events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        let mut auto_concluido = false;
        for imagem in imagens {
            let destino = self.proximo_destino();
            match Self::salvar_png(&imagem, &destino) {
                Ok(()) => {
                    self.ultima_msg = Some(format!("captura salva em {}", destino.display()));
                    auto_concluido = self.auto.is_some();
                }
                Err(e) => self.ultima_msg = Some(format!("falha na captura: {e}")),
            }
        }
        auto_concluido
    }

    fn proximo_destino(&mut self) -> PathBuf {
        if let Some(caminho) = &self.auto {
            return caminho.clone();
        }
        self.contador += 1;
        PathBuf::from(format!("nur-screenshot-{:03}.png", self.contador))
    }

    fn salvar_png(imagem: &egui::ColorImage, destino: &Path) -> Result<(), image::ImageError> {
        let [largura, altura] = imagem.size;
        image::save_buffer(
            destino,
            imagem.as_raw(),
            largura as u32,
            altura as u32,
            image::ExtendedColorType::Rgba8,
        )
    }
}

impl Default for Capturador {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
