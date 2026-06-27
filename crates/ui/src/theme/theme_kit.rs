//! Preferência de tema e instalação dos `Visuals` no contexto egui.

use crate::theme::Palette;

/// Preferência de tema escolhida pelo usuário (persistível).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    /// Tema claro.
    Light,
    /// Tema escuro.
    Dark,
}

impl ThemePreference {
    /// Alterna entre claro e escuro.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    /// Paleta correspondente a esta preferência.
    #[must_use]
    pub const fn palette(self) -> Palette {
        match self {
            Self::Light => Palette::light(),
            Self::Dark => Palette::dark(),
        }
    }
}

/// Instala o estilo do Nur (cores) num contexto egui.
pub struct ThemeKit;

impl ThemeKit {
    /// Aplica os `Visuals` derivados da preferência ao contexto.
    pub fn install(ctx: &egui::Context, pref: ThemePreference) {
        let palette = pref.palette();
        let (theme, egui_pref) = match pref {
            ThemePreference::Light => (egui::Theme::Light, egui::ThemePreference::Light),
            ThemePreference::Dark => (egui::Theme::Dark, egui::ThemePreference::Dark),
        };
        let mut visuals = match theme {
            egui::Theme::Light => egui::Visuals::light(),
            egui::Theme::Dark => egui::Visuals::dark(),
        };
        visuals.panel_fill = palette.background();
        visuals.window_fill = palette.surface();
        visuals.override_text_color = Some(palette.text());
        // Controles (combos/campos) com a cor de "control" (1 nível acima do
        // card), borda sutil e cantos arredondados — espelhando o protótipo.
        let radius = egui::CornerRadius::same(8);
        let stroke = egui::Stroke::new(1.0, palette.border());
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.bg_fill = palette.control();
            widget.weak_bg_fill = palette.control();
            widget.bg_stroke = stroke;
            widget.corner_radius = radius;
        }
        // Trava o tema (sem seguir o sistema, senão o toggle não "pega") e
        // instala os Visuals no slot do tema correspondente.
        ctx.set_visuals_of(theme, visuals);
        ctx.set_theme(egui_pref);
        // Altura e padding confortáveis para selects/inputs (~px-3 py-2.5).
        ctx.all_styles_mut(|style| {
            style.spacing.interact_size.y = 34.0;
            style.spacing.button_padding = egui::vec2(12.0, 9.0);
        });
    }
}

#[cfg(test)]
mod tests;
