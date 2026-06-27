//! Componente: campo de texto rotulado.

use crate::theme::Palette;

/// Campo de texto de linha única com rótulo pequeno acima.
pub(crate) struct LabeledInput;

impl LabeledInput {
    /// Desenha rótulo + input (com padding consistente).
    pub(crate) fn show(ui: &mut egui::Ui, palette: Palette, label: &str, value: &mut String) {
        ui.label(egui::RichText::new(label).color(palette.muted()).size(11.0));
        ui.add_space(3.0);
        ui.add(
            egui::TextEdit::singleline(value)
                .margin(egui::Margin::symmetric(12, 9))
                .desired_width(f32::INFINITY),
        );
    }
}
