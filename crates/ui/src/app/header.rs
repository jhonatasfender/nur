//! Header do card: logo, título, toggle de tema e botão de captura.

use super::NurApp;
use crate::theme::Palette;

impl NurApp {
    pub(super) fn header(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let resp = ui
            .horizontal(|ui| {
                Self::app_icon(ui, palette);
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Nur")
                            .color(palette.text())
                            .size(18.0)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("Formatar & criar boot a partir de ISO")
                            .color(palette.muted())
                            .size(12.0),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.theme_toggle(ui, palette);
                    ui.add_space(6.0);
                    self.screenshot_button(ui, palette);
                });
            })
            .response;
        // Arrastar a janela pelo header — exceto sobre o botão de tema (direita),
        // senão o arraste rouba o clique e o toggle claro/escuro não funciona.
        let drag_rect = egui::Rect::from_min_max(
            resp.rect.min,
            egui::pos2(resp.rect.max.x - 100.0, resp.rect.max.y),
        );
        if ui
            .interact(drag_rect, egui::Id::new("titlebar"), egui::Sense::drag())
            .drag_started()
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }

    // Logo: quadrado de destaque com um glifo simples de pendrive.
    fn app_icon(ui: &mut egui::Ui, palette: Palette) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, egui::CornerRadius::same(10), palette.accent());
        let center = rect.center();
        let body =
            egui::Rect::from_center_size(center + egui::vec2(0.0, 3.0), egui::vec2(12.0, 16.0));
        p.rect_filled(body, egui::CornerRadius::same(3), palette.on_accent());
        let cap =
            egui::Rect::from_center_size(center - egui::vec2(0.0, 10.0), egui::vec2(7.0, 5.0));
        p.rect_filled(cap, egui::CornerRadius::same(1), palette.on_accent());
    }

    fn theme_toggle(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let icon = if self.theme == crate::theme::ThemePreference::Dark {
            "\u{2600}"
        } else {
            "\u{1F319}"
        };
        let button = egui::Button::new(egui::RichText::new(icon).size(15.0))
            .fill(palette.control())
            .corner_radius(egui::CornerRadius::same(8));
        if ui.add_sized([36.0, 36.0], button).clicked() {
            self.theme = self.theme.toggle();
            self.theme_installed = false;
        }
    }

    fn screenshot_button(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let button = egui::Button::new(egui::RichText::new("\u{1F4F7}").size(14.0))
            .fill(palette.control())
            .corner_radius(egui::CornerRadius::same(8));
        if ui
            .add_sized([36.0, 36.0], button)
            .on_hover_text("Capturar tela (ou F12)")
            .clicked()
        {
            self.capturer.capture_now(ui.ctx());
        }
    }
}
