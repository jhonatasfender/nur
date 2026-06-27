//! Preferência de tema e instalação dos `Visuals` no contexto egui.

use crate::theme::Palette;

/// Preferência de tema escolhida pelo usuário (persistível).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    /// Tema claro.
    Claro,
    /// Tema escuro.
    Escuro,
}

impl ThemePreference {
    /// Alterna entre claro e escuro.
    #[must_use]
    pub const fn alternar(self) -> Self {
        match self {
            Self::Claro => Self::Escuro,
            Self::Escuro => Self::Claro,
        }
    }

    /// Paleta correspondente a esta preferência.
    #[must_use]
    pub const fn palette(self) -> Palette {
        match self {
            Self::Claro => Palette::clara(),
            Self::Escuro => Palette::escura(),
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
            ThemePreference::Claro => egui::Visuals::light(),
            ThemePreference::Escuro => egui::Visuals::dark(),
        };
        visuals.panel_fill = palette.fundo;
        visuals.window_fill = palette.superficie;
        visuals.override_text_color = Some(palette.texto);
        ctx.set_visuals(visuals);
    }
}

#[cfg(test)]
mod tests;
