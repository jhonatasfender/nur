//! Paletas de cores claro/escuro, espelhando os protótipos HTML.

use egui::Color32;

/// Conjunto de cores de um tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Cor de fundo da janela.
    pub fundo: Color32,
    /// Cor das superfícies/cards.
    pub superficie: Color32,
    /// Cor de texto primário.
    pub texto: Color32,
    /// Acento destrutivo (vermelho).
    pub destrutivo: Color32,
    /// Acento de sucesso (verde).
    pub sucesso: Color32,
}

impl Palette {
    /// Tema claro (fundo cinza-claro, superfície branca).
    #[must_use]
    pub const fn clara() -> Self {
        Self {
            fundo: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            superficie: Color32::WHITE,
            texto: Color32::from_rgb(0x11, 0x18, 0x27),
            destrutivo: Color32::from_rgb(0xDC, 0x26, 0x26),
            sucesso: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }

    /// Tema escuro (fundo quase preto, superfície cinza-escuro).
    #[must_use]
    pub const fn escura() -> Self {
        Self {
            fundo: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            superficie: Color32::from_rgb(0x11, 0x18, 0x27),
            texto: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            destrutivo: Color32::from_rgb(0xDC, 0x26, 0x26),
            sucesso: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }
}

#[cfg(test)]
mod tests;
