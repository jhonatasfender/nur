//! Aplicação egui do Nur (presenter; consome portas via `Arc<dyn _>`).

use crate::capture::Capturer;
use crate::theme::{Fonts, ThemeKit, ThemePreference};
use application::ports::UiState;
use std::sync::Arc;

mod draw;
mod modal;
mod options;
mod status;
#[cfg(test)]
mod tests;

/// O que o usuário quer fazer com o dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Criar um pendrive bootável a partir de uma ISO.
    Boot,
    /// Apenas formatar o dispositivo.
    Format,
}

/// Fase da operação em andamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Nada em andamento.
    Idle,
    /// Preparando o dispositivo (indeterminado).
    Preparing,
    /// Gravando/formatando (determinado).
    Working,
    /// Verificando após gravar.
    Verifying,
    /// Concluído com sucesso.
    Done,
}

/// App egui do Nur. Lê o estado por uma porta injetada.
pub struct NurApp {
    state: Arc<dyn UiState>,
    theme: ThemePreference,
    theme_installed: bool,
    fonts_installed: bool,
    capturer: Capturer,
    selected: Option<usize>,
    mode: Mode,
    iso_selected: bool,
    partition: usize,
    target: usize,
    filesystem: usize,
    label: String,
    quick_format: bool,
    modal_open: bool,
    confirm_text: String,
    phase: Phase,
    progress: f32,
}

impl NurApp {
    /// Cria o app com o estado injetado (tema padrão: escuro).
    #[must_use]
    pub fn new(state: Arc<dyn UiState>) -> Self {
        let mut app = Self {
            state,
            theme: ThemePreference::Dark,
            theme_installed: false,
            fonts_installed: false,
            capturer: Capturer::new(),
            selected: None,
            mode: Mode::Boot,
            iso_selected: false,
            partition: 0,
            target: 0,
            filesystem: 0,
            label: "BOOTUSB".to_owned(),
            quick_format: true,
            modal_open: false,
            confirm_text: String::new(),
            phase: Phase::Idle,
            progress: 0.0,
        };
        app.seed_demo();
        app
    }

    // Estado inicial para captura/validação visual (env `NUR_DEMO`); no-op em produção.
    fn seed_demo(&mut self) {
        if std::env::var("NUR_THEME").as_deref() == Ok("light") {
            self.theme = ThemePreference::Light;
        }
        match std::env::var("NUR_DEMO").as_deref() {
            Ok("ready") => {
                self.selected = Some(0);
                self.iso_selected = true;
            }
            Ok("modal") => {
                self.selected = Some(0);
                self.iso_selected = true;
                self.modal_open = true;
                "APAGAR".clone_into(&mut self.confirm_text);
            }
            Ok("running") => {
                self.selected = Some(0);
                self.iso_selected = true;
                self.phase = Phase::Working;
                self.progress = 0.42;
            }
            _ => {}
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

    // Avança a simulação de progresso enquanto há operação em andamento.
    fn tick(&mut self, ctx: &egui::Context) {
        if matches!(self.phase, Phase::Idle | Phase::Done) {
            return;
        }
        ctx.request_repaint();
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        match self.phase {
            Phase::Preparing => {
                self.progress += dt * 0.8;
                if self.progress >= 1.0 {
                    self.progress = 0.0;
                    self.phase = Phase::Working;
                }
            }
            Phase::Working => {
                self.progress += dt * 0.35;
                if self.progress >= 1.0 {
                    self.progress = 1.0;
                    self.phase = Phase::Verifying;
                }
            }
            Phase::Verifying => {
                self.progress += dt * 1.2;
                if self.progress >= 1.0 {
                    self.phase = Phase::Done;
                }
            }
            Phase::Idle | Phase::Done => {}
        }
    }
}

impl eframe::App for NurApp {
    /// Cor de fundo da janela = backdrop do tema.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.palette().background().to_normalized_gamma_f32()
    }

    /// Instala fonte/tema, trata captura e avança o progresso.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_installed {
            Fonts::install(ctx);
            self.fonts_installed = true;
        }
        if !self.theme_installed {
            ThemeKit::install(ctx, self.theme);
            self.theme_installed = true;
        }
        self.tick(ctx);
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
