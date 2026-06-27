//! Modal de confirmação ("digite APAGAR") e início da operação.

use super::{NurApp, Phase};
use crate::theme::Palette;

impl NurApp {
    pub(super) fn open_confirm(&mut self) {
        self.modal_open = true;
        self.confirm_text.clear();
    }

    pub(super) fn draw_modal(&mut self, ui: &mut egui::Ui) {
        if !self.modal_open {
            return;
        }
        let palette = self.theme.palette();
        let screen = ui.ctx().viewport_rect();
        ui.painter()
            .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(150));
        let device_path = self
            .selected
            .and_then(|i| self.state.devices().get(i).map(|d| d.path().to_owned()))
            .unwrap_or_default();
        let (mut cancel, mut confirm) = (false, false);
        egui::Window::new("confirm")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .frame(
                egui::Frame::NONE
                    .fill(palette.surface())
                    .stroke(egui::Stroke::new(1.0, palette.border()))
                    .corner_radius(egui::CornerRadius::same(16))
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ui.ctx(), |ui| {
                ui.set_width(300.0);
                Self::danger_badge(ui, palette);
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Apagar tudo deste pendrive?").color(palette.text()).size(17.0).strong());
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Esta ação é irreversível. Todos os arquivos de {device_path} serão perdidos."
                    ))
                    .color(palette.muted())
                    .size(13.0),
                );
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Digite APAGAR para confirmar").color(palette.muted()).size(11.0));
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.confirm_text)
                        .hint_text("APAGAR")
                        .margin(egui::Margin::symmetric(12, 9))
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(14.0);
                let ok = self.confirm_text.trim().eq_ignore_ascii_case("APAGAR");
                ui.columns(2, |cols| {
                    let cancel_btn = egui::Button::new(egui::RichText::new("Cancelar").color(palette.text()).size(13.0).strong())
                        .fill(palette.control())
                        .corner_radius(egui::CornerRadius::same(8));
                    if cols[0].add_sized([cols[0].available_width(), 38.0], cancel_btn).clicked() {
                        cancel = true;
                    }
                    let confirm_btn = egui::Button::new(egui::RichText::new("Apagar e gravar").color(egui::Color32::WHITE).size(13.0).strong())
                        .fill(egui::Color32::from_rgb(0xDC, 0x26, 0x26))
                        .corner_radius(egui::CornerRadius::same(8));
                    let resp = cols[1].add_enabled_ui(ok, |ui| {
                        ui.add_sized([ui.available_width(), 38.0], confirm_btn)
                    });
                    if resp.inner.clicked() {
                        confirm = true;
                    }
                });
            });
        if cancel {
            self.modal_open = false;
        }
        if confirm {
            self.modal_open = false;
            self.phase = Phase::Preparing;
            self.progress = 0.0;
        }
    }

    // Selo circular vermelho com ícone de alerta.
    fn danger_badge(ui: &mut egui::Ui, palette: Palette) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
        let d = palette.destructive();
        let bg = egui::Color32::from_rgba_unmultiplied(d.r(), d.g(), d.b(), 40);
        let painter = ui.painter();
        painter.circle_filled(rect.center(), 22.0, bg);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{26A0}",
            egui::FontId::proportional(22.0),
            d,
        );
    }
}
