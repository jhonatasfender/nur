//! Paletas de cores claro/escuro, espelhando os protótipos HTML.

use egui::Color32;

/// Conjunto de cores de um tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    background: Color32,
    surface: Color32,
    text: Color32,
    destructive: Color32,
    success: Color32,
}

impl Palette {
    /// Tema claro (fundo cinza-claro, superfície branca).
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            surface: Color32::WHITE,
            text: Color32::from_rgb(0x11, 0x18, 0x27),
            destructive: Color32::from_rgb(0xDC, 0x26, 0x26),
            success: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }

    /// Tema escuro (fundo quase preto, superfície cinza-escuro).
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            surface: Color32::from_rgb(0x11, 0x18, 0x27),
            text: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            destructive: Color32::from_rgb(0xDC, 0x26, 0x26),
            success: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }

    /// Cor de fundo da janela.
    #[must_use]
    pub const fn background(self) -> Color32 {
        self.background
    }

    /// Cor das superfícies/cards.
    #[must_use]
    pub const fn surface(self) -> Color32 {
        self.surface
    }

    /// Cor de texto primário.
    #[must_use]
    pub const fn text(self) -> Color32 {
        self.text
    }

    /// Acento destrutivo (vermelho).
    #[must_use]
    pub const fn destructive(self) -> Color32 {
        self.destructive
    }

    /// Acento de sucesso (verde).
    #[must_use]
    pub const fn success(self) -> Color32 {
        self.success
    }
}

#[cfg(test)]
mod tests;
