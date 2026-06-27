//! Componente: rótulo de seção (maiúsculas, pequeno, atenuado).

use crate::theme::Palette;

/// Rótulo de campo no estilo "field label" do protótipo.
pub(crate) struct FieldLabel;

impl FieldLabel {
    /// Desenha o rótulo e um pequeno espaço abaixo.
    pub(crate) fn show(ui: &mut egui::Ui, palette: Palette, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .color(palette.muted())
                .size(11.0)
                .strong(),
        );
        ui.add_space(5.0);
    }
}
