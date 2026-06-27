//! Aplicação egui do Nur (presenter; consome portas via Arc<dyn _>).

use crate::captura::Capturador;
use crate::theme::{ThemeKit, ThemePreference};
use application::ports::UiState;
use std::sync::Arc;

/// App egui do Nur. Lê o estado por uma porta injetada.
pub struct NurApp {
    estado: Arc<dyn UiState>,
    tema: ThemePreference,
    tema_instalado: bool,
    capturador: Capturador,
}

impl NurApp {
    /// Cria o app com o estado injetado (tema padrão: escuro).
    #[must_use]
    pub fn new(estado: Arc<dyn UiState>) -> Self {
        Self {
            estado,
            tema: ThemePreference::Escuro,
            tema_instalado: false,
            capturador: Capturador::new(),
        }
    }

    /// Builder: define a preferência de tema inicial.
    #[must_use]
    pub fn com_tema(mut self, pref: ThemePreference) -> Self {
        self.tema = pref;
        self
    }

    #[cfg(test)]
    pub(crate) fn tema(&self) -> ThemePreference {
        self.tema
    }
}

impl eframe::App for NurApp {
    /// Instala o tema antes de redesenhar (eframe 0.35: logic recebe ctx).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.tema_instalado {
            ThemeKit::install(ctx, self.tema);
            self.tema_instalado = true;
        }
        // Captura de tela (F12 ou modo automático). Fecha ao concluir a captura automática.
        if self.capturador.processar(ctx) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Desenha o painel central (eframe 0.35: ui recebe &mut Ui).
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Nur");
        ui.label("Formatador de Pendrive & Criador de Boot");
        if ui.button("Alternar tema").clicked() {
            self.tema = self.tema.alternar();
            self.tema_instalado = false;
        }
        ui.separator();
        ui.label("Dispositivos detectados:");
        for d in self.estado.dispositivos() {
            ui.label(d.descricao);
        }
        ui.separator();
        ui.label("Pressione F12 para capturar a tela.");
        if let Some(msg) = self.capturador.mensagem() {
            ui.label(msg);
        }
    }
}

#[cfg(test)]
mod tests;
