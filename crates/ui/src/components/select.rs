//! Componente: select rotulado (combo com rótulo pequeno acima).

use crate::theme::Palette;

/// Combo de opções (`&[&str]`) com um rótulo pequeno acima.
pub(crate) struct LabeledSelect;

impl LabeledSelect {
    /// Desenha rótulo + combo; `value` é o índice selecionado.
    pub(crate) fn show(
        ui: &mut egui::Ui,
        palette: Palette,
        id: &str,
        label: &str,
        options: &[&str],
        value: &mut usize,
    ) {
        ui.label(egui::RichText::new(label).color(palette.muted()).size(11.0));
        ui.add_space(3.0);
        egui::ComboBox::from_id_salt(id)
            .selected_text(options[*value])
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (i, option) in options.iter().enumerate() {
                    ui.selectable_value(value, i, *option);
                }
            });
    }
}
