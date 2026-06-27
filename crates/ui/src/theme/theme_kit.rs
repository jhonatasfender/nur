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
        let mut visuals = match pref {
            ThemePreference::Light => egui::Visuals::light(),
            ThemePreference::Dark => egui::Visuals::dark(),
        };
        visuals.panel_fill = palette.background();
        visuals.window_fill = palette.surface();
        visuals.override_text_color = Some(palette.text());
        ctx.set_visuals(visuals);
    }
}

#[cfg(test)]
mod tests;
