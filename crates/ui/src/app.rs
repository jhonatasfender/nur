//! Aplicação egui do Nur (presenter; consome portas via `Arc<dyn _>`).

use crate::capture::Capturer;
use crate::theme::{ThemeKit, ThemePreference};
use application::ports::UiState;
use std::sync::Arc;

/// App egui do Nur. Lê o estado por uma porta injetada.
pub struct NurApp {
    state: Arc<dyn UiState>,
    theme: ThemePreference,
    theme_installed: bool,
    capturer: Capturer,
}

impl NurApp {
    /// Cria o app com o estado injetado (tema padrão: escuro).
    #[must_use]
    pub fn new(state: Arc<dyn UiState>) -> Self {
        Self {
            state,
            theme: ThemePreference::Dark,
            theme_installed: false,
            capturer: Capturer::new(),
        }
    }

    /// Builder: define a preferência de tema inicial.
    #[must_use]
    pub fn with_theme(mut self, pref: ThemePreference) -> Self {
        self.theme = pref;
        self
    }

    #[cfg(test)]
    pub(crate) fn theme(&self) -> ThemePreference {
        self.theme
    }
}

impl eframe::App for NurApp {
    /// Instala o tema antes de redesenhar (eframe 0.35: logic recebe ctx).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_installed {
            ThemeKit::install(ctx, self.theme);
            self.theme_installed = true;
        }
        // Captura de tela (F12 ou modo automático). Fecha ao concluir a captura automática.
        if self.capturer.process(ctx) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Desenha o painel central (eframe 0.35: ui recebe &mut Ui).
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Nur");
        ui.label("Formatador de Pendrive & Criador de Boot");
        if ui.button("Alternar tema").clicked() {
            self.theme = self.theme.toggle();
            self.theme_installed = false;
        }
        ui.separator();
        ui.label("Dispositivos detectados:");
        for d in self.state.devices() {
            ui.label(d.description());
        }
        ui.separator();
        ui.label("Pressione F12 para capturar a tela.");
        if let Some(msg) = self.capturer.message() {
            ui.label(msg);
        }
    }
}

#[cfg(test)]
mod tests;
