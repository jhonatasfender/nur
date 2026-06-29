//! Seção de status (barra de progresso) e footer (Fechar/Iniciar/Cancelar).

use super::{Mode, NurApp};
use crate::components::{PrimaryButton, SecondaryButton};
use crate::theme::Palette;
use application::ports::WriteState;
use domain::{IsoKind, VolumeLabel};

impl NurApp {
    // Há uma operação destrutiva em andamento?
    pub(super) fn in_progress(&self) -> bool {
        matches!(
            self.state.write_state(),
            WriteState::Preparing | WriteState::Writing { .. } | WriteState::Verifying { .. }
        )
    }

    // Pronto para iniciar? Boot exige ISO isohybrid; Format exige rótulo válido.
    pub(super) fn ready(&self) -> bool {
        let mode_ok = match self.mode {
            Mode::Format => VolumeLabel::parse(&self.label).is_ok(),
            Mode::Boot => self
                .state
                .selected_iso()
                .is_some_and(|s| s.kind() == IsoKind::Isohybrid),
        };
        self.selected.is_some() && mode_ok && !self.in_progress()
    }

    // Fração 0..1 da barra a partir do estado real.
    fn progress_fraction(&self) -> f32 {
        match self.state.write_state() {
            WriteState::Writing { done, total } | WriteState::Verifying { done, total } => {
                if total > 0 {
                    done as f32 / total as f32
                } else {
                    0.0
                }
            }
            WriteState::Done => 1.0,
            _ => 0.0,
        }
    }

    pub(super) fn status_section(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let state = self.state.write_state();
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("STATUS")
                    .color(palette.muted())
                    .size(11.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let pct = match &state {
                    WriteState::Idle => "Pronto".to_owned(),
                    WriteState::Done => "Concluído".to_owned(),
                    WriteState::Preparing => String::new(),
                    _ => format!("{}%", (self.progress_fraction() * 100.0) as i32),
                };
                let color = if state == WriteState::Done {
                    palette.success()
                } else {
                    palette.muted()
                };
                ui.label(egui::RichText::new(pct).color(color).size(12.0).strong());
            });
        });
        ui.add_space(6.0);
        self.progress_bar(ui, palette, &state);
        ui.add_space(6.0);
        let (text, color) = self.status_text(&state, palette);
        ui.label(egui::RichText::new(text).color(color).size(12.0));
    }

    fn progress_bar(&self, ui: &mut egui::Ui, palette: Palette, state: &WriteState) {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), egui::Sense::hover());
        let painter = ui.painter();
        let radius = egui::CornerRadius::same(4);
        painter.rect_filled(rect, radius, palette.control());
        let frac = self.progress_fraction().clamp(0.0, 1.0);
        if frac > 0.0 {
            let mut fill = rect;
            fill.set_width(rect.width() * frac);
            let color = if *state == WriteState::Done {
                palette.success()
            } else {
                palette.accent()
            };
            painter.rect_filled(fill, radius, color);
        }
    }

    fn status_text(&self, state: &WriteState, palette: Palette) -> (String, egui::Color32) {
        let muted = palette.muted();
        match state {
            WriteState::Idle if self.ready() => ("Pronto para iniciar.".to_owned(), muted),
            WriteState::Idle => ("Selecione um dispositivo para começar.".to_owned(), muted),
            WriteState::Preparing if self.mode == Mode::Format => {
                ("Formatando\u{2026}".to_owned(), muted)
            }
            WriteState::Preparing => ("Preparando dispositivo\u{2026}".to_owned(), muted),
            WriteState::Writing { .. } if self.mode == Mode::Boot => {
                ("Gravando imagem\u{2026}".to_owned(), muted)
            }
            WriteState::Writing { .. } => ("Formatando\u{2026}".to_owned(), muted),
            WriteState::Verifying { .. } => ("Verificando\u{2026}".to_owned(), muted),
            WriteState::Done if self.mode == Mode::Boot => {
                ("Pendrive bootável pronto!".to_owned(), palette.success())
            }
            WriteState::Done => ("Formatação concluída!".to_owned(), palette.success()),
            WriteState::Failed(msg) => (msg.clone(), palette.destructive()),
            WriteState::Cancelled => (
                "Cancelado \u{2014} pendrive incompleto, regrave.".to_owned(),
                palette.destructive(),
            ),
        }
    }

    pub(super) fn footer(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(12.0);
        let total = ui.available_width();
        let close_w = (total - 12.0) * 0.4;
        let start_w = (total - 12.0) * 0.6;
        let running = self.in_progress();
        let ready = self.ready();
        ui.horizontal(|ui| {
            if SecondaryButton::show(ui, palette, "Fechar", close_w) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if running {
                if PrimaryButton::show(ui, palette, "Cancelar", start_w, true) {
                    self.commands.cancel();
                }
            } else if PrimaryButton::show(ui, palette, "Iniciar", start_w, ready) {
                self.open_confirm();
            }
        });
    }
}
