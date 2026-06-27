//! Seção de status (barra de progresso) e footer (Fechar/Iniciar).

use super::{Mode, NurApp, Phase};
use crate::components::{PrimaryButton, SecondaryButton};
use crate::theme::Palette;

impl NurApp {
    // Pronto para iniciar? (dispositivo + ISO quando em modo bootável)
    pub(super) fn ready(&self) -> bool {
        self.selected.is_some() && (self.mode == Mode::Format || self.iso_selected)
    }

    pub(super) fn status_section(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("STATUS")
                    .color(palette.muted())
                    .size(11.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let pct = match self.phase {
                    Phase::Idle => "Pronto".to_owned(),
                    Phase::Done => "Concluído".to_owned(),
                    Phase::Preparing => String::new(),
                    _ => format!("{}%", (self.progress * 100.0) as i32),
                };
                let color = if self.phase == Phase::Done {
                    palette.success()
                } else {
                    palette.muted()
                };
                ui.label(egui::RichText::new(pct).color(color).size(12.0).strong());
            });
        });
        ui.add_space(6.0);
        self.progress_bar(ui, palette);
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(self.status_text())
                .color(palette.muted())
                .size(12.0),
        );
    }

    fn progress_bar(&self, ui: &mut egui::Ui, palette: Palette) {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), egui::Sense::hover());
        let painter = ui.painter();
        let radius = egui::CornerRadius::same(4);
        painter.rect_filled(rect, radius, palette.control());
        let frac = self.progress.clamp(0.0, 1.0);
        if frac > 0.0 {
            let mut fill = rect;
            fill.set_width(rect.width() * frac);
            let color = if self.phase == Phase::Done {
                palette.success()
            } else {
                palette.accent()
            };
            painter.rect_filled(fill, radius, color);
        }
    }

    fn status_text(&self) -> &'static str {
        match self.phase {
            Phase::Idle if self.ready() => "Pronto para iniciar.",
            Phase::Idle => "Selecione um dispositivo para começar.",
            Phase::Preparing => "Preparando dispositivo\u{2026}",
            Phase::Working if self.mode == Mode::Boot => "Gravando imagem\u{2026}",
            Phase::Working => "Formatando\u{2026}",
            Phase::Verifying => "Verificando\u{2026}",
            Phase::Done if self.mode == Mode::Boot => "Pendrive bootável pronto!",
            Phase::Done => "Formatação concluída!",
        }
    }

    pub(super) fn footer(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(12.0);
        let total = ui.available_width();
        let close_w = (total - 12.0) * 0.4;
        let start_w = (total - 12.0) * 0.6;
        let ready = self.ready();
        ui.horizontal(|ui| {
            if SecondaryButton::show(ui, palette, "Fechar", close_w) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if PrimaryButton::show(ui, palette, "Iniciar", start_w, ready) {
                self.open_confirm();
            }
        });
    }
}
