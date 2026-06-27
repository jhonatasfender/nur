//! Componentes de botão (primário/accent, secundário/control, perigo/vermelho).

use crate::theme::Palette;

const HEIGHT: f32 = 38.0;
const RADIUS: u8 = 8;

/// Botão de destaque (cor de accent).
pub(crate) struct PrimaryButton;

impl PrimaryButton {
    /// Desenha o botão com `width`, habilitado conforme `enabled`. `true` se clicado.
    pub(crate) fn show(
        ui: &mut egui::Ui,
        palette: Palette,
        text: &str,
        width: f32,
        enabled: bool,
    ) -> bool {
        let button = egui::Button::new(
            egui::RichText::new(text)
                .color(palette.on_accent())
                .size(13.0)
                .strong(),
        )
        .fill(palette.accent())
        .corner_radius(egui::CornerRadius::same(RADIUS));
        ui.add_enabled_ui(enabled, |ui| ui.add_sized([width, HEIGHT], button))
            .inner
            .clicked()
    }
}

/// Botão neutro (cor de control).
pub(crate) struct SecondaryButton;

impl SecondaryButton {
    /// Desenha o botão com `width`. `true` se clicado.
    pub(crate) fn show(ui: &mut egui::Ui, palette: Palette, text: &str, width: f32) -> bool {
        let button = egui::Button::new(
            egui::RichText::new(text)
                .color(palette.text())
                .size(13.0)
                .strong(),
        )
        .fill(palette.control())
        .corner_radius(egui::CornerRadius::same(RADIUS));
        ui.add_sized([width, HEIGHT], button).clicked()
    }
}

/// Botão destrutivo (vermelho).
pub(crate) struct DangerButton;

impl DangerButton {
    /// Desenha o botão com `width`, habilitado conforme `enabled`. `true` se clicado.
    pub(crate) fn show(ui: &mut egui::Ui, text: &str, width: f32, enabled: bool) -> bool {
        let button = egui::Button::new(
            egui::RichText::new(text)
                .color(egui::Color32::WHITE)
                .size(13.0)
                .strong(),
        )
        .fill(egui::Color32::from_rgb(0xDC, 0x26, 0x26))
        .corner_radius(egui::CornerRadius::same(RADIUS));
        ui.add_enabled_ui(enabled, |ui| ui.add_sized([width, HEIGHT], button))
            .inner
            .clicked()
    }
}
