//! Aplicação egui do Nur (presenter; consome portas via `Arc<dyn _>`).

use crate::capture::Capturer;
use crate::theme::{Fonts, ThemeKit, ThemePreference};
use application::ports::{ScreenshotWriter, UiCommands, UiState};
use std::path::PathBuf;
use std::sync::Arc;

mod demo;
mod draw;
mod header;
mod modal;
mod options;
mod status;
#[cfg(test)]
mod tests;

pub use demo::DemoScenario;

/// O que o usuário quer fazer com o dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Criar um pendrive bootável a partir de uma ISO.
    Boot,
    /// Apenas formatar o dispositivo.
    Format,
}

/// App egui do Nur. Lê o estado por uma porta injetada.
pub struct NurApp {
    state: Arc<dyn UiState>,
    commands: Arc<dyn UiCommands>,
    theme: ThemePreference,
    theme_installed: bool,
    fonts_installed: bool,
    capturer: Capturer,
    selected: Option<usize>,
    mode: Mode,
    partition: usize,
    target: usize,
    filesystem: usize,
    label: String,
    quick_format: bool,
    modal_open: bool,
    confirm_text: String,
}

impl NurApp {
    /// Cria o app com o estado, os comandos e o gravador de capturas injetados
    /// (tema escuro por padrão).
    #[must_use]
    pub fn new(
        state: Arc<dyn UiState>,
        commands: Arc<dyn UiCommands>,
        screenshots: Arc<dyn ScreenshotWriter>,
    ) -> Self {
        Self {
            state,
            commands,
            theme: ThemePreference::Dark,
            theme_installed: false,
            fonts_installed: false,
            capturer: Capturer::new(screenshots, None),
            selected: None,
            mode: Mode::Boot,
            partition: 0,
            target: 0,
            filesystem: 0,
            label: "BOOTUSB".to_owned(),
            quick_format: true,
            modal_open: false,
            confirm_text: String::new(),
        }
    }

    /// Builder: define a preferência de tema inicial.
    #[must_use]
    pub fn with_theme(mut self, pref: ThemePreference) -> Self {
        self.theme = pref;
        self
    }

    /// Builder: define o destino de captura automática (modo headless).
    #[must_use]
    pub fn with_capture_path(mut self, dest: Option<PathBuf>) -> Self {
        self.capturer.set_auto(dest);
        self
    }

    #[cfg(test)]
    pub(crate) fn theme(&self) -> ThemePreference {
        self.theme
    }
}

impl eframe::App for NurApp {
    /// Janela transparente: o card arredondado é pintado em `ui`; fora dele
    /// fica transparente, dando à janela cantos redondos.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    /// Instala fonte/tema e trata captura. O progresso é dirigido pelo estado
    /// real (a ponte do app repinta a UI ao atualizar `write_state`).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_installed {
            Fonts::install(ctx);
            self.fonts_installed = true;
        }
        if !self.theme_installed {
            ThemeKit::install(ctx, self.theme);
            self.theme_installed = true;
        }
        if self.capturer.process(ctx) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Desenha o painel principal (card central) e o modal.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw_card(ui);
        self.draw_modal(ui);
    }
}
