//! Card central e seções de topo (header, dispositivo, modo).

use super::{Mode, NurApp};
use crate::components::FieldLabel;
use crate::theme::Palette;

impl NurApp {
    /// Card central (superfície arredondada) com todas as seções.
    pub(super) fn draw_card(&mut self, ui: &mut egui::Ui) {
        let palette = self.theme.palette();
        // A janela é o próprio card: pinta um retângulo arredondado cobrindo a
        // viewport. Como a janela é transparente, os cantos ficam redondos.
        let rect = ui.ctx().viewport_rect();
        let radius = egui::CornerRadius::same(16);
        let painter = ui.painter().clone();
        painter.rect_filled(rect, radius, palette.surface());
        painter.rect_stroke(
            rect.shrink(0.5),
            radius,
            egui::Stroke::new(1.0, palette.border()),
            egui::StrokeKind::Inside,
        );
        // Conteúdo com padding interno (o card ocupa a janela inteira).
        let content = egui::Frame::NONE
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                self.header(ui, palette);
                ui.add_space(18.0);
                self.device_selector(ui, palette);
                ui.add_space(18.0);
                self.mode_selector(ui, palette);
                ui.add_space(18.0);
                self.iso_section(ui, palette);
                self.options_section(ui, palette);
                ui.add_space(18.0);
                self.status_section(ui, palette);
                ui.add_space(18.0);
                self.footer(ui, palette);
            });
        // A janela acompanha a altura do conteúdo (cresce/encolhe ao expandir
        // seções como a ISO ou o aviso de dispositivo).
        let desired_h = content.response.rect.height();
        if (rect.height() - desired_h).abs() > 1.0 {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    rect.width(),
                    desired_h,
                )));
        }
    }

    fn device_selector(&mut self, ui: &mut egui::Ui, palette: Palette) {
        FieldLabel::show(ui, palette, "DISPOSITIVO");
        let devices = self.state.devices();
        let selected_text = self.selected.and_then(|i| devices.get(i)).map_or_else(
            || "— Selecione o pendrive —".to_owned(),
            |d| d.description().to_owned(),
        );
        egui::ComboBox::from_id_salt("device")
            .selected_text(selected_text)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (i, device) in devices.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, Some(i), device.description());
                }
            });
        if self.selected.is_some() {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("\u{26A0} Todos os dados deste dispositivo serão apagados.")
                    .color(palette.destructive())
                    .size(12.0),
            );
        }
    }

    fn mode_selector(&mut self, ui: &mut egui::Ui, palette: Palette) {
        FieldLabel::show(ui, palette, "O QUE DESEJA FAZER?");
        egui::Frame::NONE
            .fill(palette.control())
            .stroke(egui::Stroke::new(1.0, palette.border()))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                let full_w = ui.available_width();
                let half = full_w / 2.0;
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(full_w, 30.0), egui::Sense::hover());
                // Pílula deslizante: anima a posição entre as duas opções.
                let target = if self.mode == Mode::Boot { 0.0 } else { half };
                let x = ui
                    .ctx()
                    .animate_value_with_time(egui::Id::new("mode_pill"), target, 0.18);
                let pill = egui::Rect::from_min_size(
                    rect.min + egui::vec2(x, 0.0),
                    egui::vec2(half, rect.height()),
                );
                ui.painter()
                    .rect_filled(pill, egui::CornerRadius::same(6), palette.accent());
                // Rótulos clicáveis sobre a pílula.
                let options = [
                    (Mode::Boot, "Criar bootável"),
                    (Mode::Format, "Apenas formatar"),
                ];
                for (i, (mode, text)) in options.into_iter().enumerate() {
                    let cell = egui::Rect::from_min_size(
                        rect.min + egui::vec2(i as f32 * half, 0.0),
                        egui::vec2(half, rect.height()),
                    );
                    if ui
                        .interact(cell, egui::Id::new(("mode", i)), egui::Sense::click())
                        .clicked()
                    {
                        self.mode = mode;
                    }
                    let color = if self.mode == mode {
                        palette.on_accent()
                    } else {
                        palette.muted()
                    };
                    ui.painter().text(
                        cell.center(),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::proportional(13.0),
                        color,
                    );
                }
            });
    }
}
