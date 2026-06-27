//! Componente: checkbox estilizado (caixa arredondada + check desenhado).

use crate::theme::Palette;

/// Checkbox com rótulo, desenhado à mão para padding/estilo consistentes.
pub(crate) struct Checkbox;

impl Checkbox {
    /// Desenha o checkbox; alterna `checked` ao clicar.
    pub(crate) fn show(ui: &mut egui::Ui, palette: Palette, label: &str, checked: &mut bool) {
        let box_size = 18.0;
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
        if resp.clicked() {
            *checked = !*checked;
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let painter = ui.painter();
        let box_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.center().y - box_size / 2.0),
            egui::vec2(box_size, box_size),
        );
        let radius = egui::CornerRadius::same(5);
        if *checked {
            painter.rect_filled(box_rect, radius, palette.accent());
            let center = box_rect.center();
            let check = vec![
                center + egui::vec2(-4.0, 0.0),
                center + egui::vec2(-1.0, 3.5),
                center + egui::vec2(4.5, -4.0),
            ];
            painter.add(egui::Shape::line(
                check,
                egui::Stroke::new(2.0, palette.on_accent()),
            ));
        } else {
            painter.rect_filled(box_rect, radius, palette.control());
            painter.rect_stroke(
                box_rect,
                radius,
                egui::Stroke::new(1.0, palette.border()),
                egui::StrokeKind::Inside,
            );
        }
        painter.text(
            egui::pos2(box_rect.max.x + 9.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            palette.text(),
        );
    }
}
