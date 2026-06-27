//! Paletas de cores claro/escuro, espelhando o protótipo HTML (Tailwind).

use egui::Color32;

/// Conjunto de cores de um tema (níveis de superfície + acentos).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    background: Color32,
    surface: Color32,
    control: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    destructive: Color32,
    success: Color32,
    accent: Color32,
    on_accent: Color32,
}

impl Palette {
    /// Tema claro (backdrop cinza-claro, card branco).
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Color32::from_rgb(0xF3, 0xF4, 0xF6), // gray-100
            surface: Color32::WHITE,
            control: Color32::WHITE,
            border: Color32::from_rgb(0xD1, 0xD5, 0xDB), // gray-300
            text: Color32::from_rgb(0x11, 0x18, 0x27),   // gray-900
            muted: Color32::from_rgb(0x6B, 0x72, 0x80),  // gray-500
            destructive: Color32::from_rgb(0xDC, 0x26, 0x26),
            success: Color32::from_rgb(0x16, 0xA3, 0x4A),
            accent: Color32::from_rgb(0x11, 0x18, 0x27),
            on_accent: Color32::WHITE,
        }
    }

    /// Tema escuro (backdrop quase preto, card cinza-900, inputs cinza-800).
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Color32::from_rgb(0x03, 0x07, 0x12), // gray-950
            surface: Color32::from_rgb(0x11, 0x18, 0x27),    // gray-900
            control: Color32::from_rgb(0x1F, 0x29, 0x37),    // gray-800
            border: Color32::from_rgb(0x37, 0x41, 0x51),     // gray-700
            text: Color32::from_rgb(0xF3, 0xF4, 0xF6),       // gray-100
            muted: Color32::from_rgb(0x9C, 0xA3, 0xAF),      // gray-400
            destructive: Color32::from_rgb(0xF8, 0x71, 0x71), // red-400
            success: Color32::from_rgb(0x22, 0xC5, 0x5E),    // green-500
            accent: Color32::from_rgb(0xF3, 0xF4, 0xF6),     // gray-100
            on_accent: Color32::from_rgb(0x11, 0x18, 0x27),
        }
    }

    /// Cor do backdrop (fundo da janela).
    #[must_use]
    pub const fn background(self) -> Color32 {
        self.background
    }

    /// Cor do card/superfície principal.
    #[must_use]
    pub const fn surface(self) -> Color32 {
        self.surface
    }

    /// Cor de controles (inputs/selects/botões neutros).
    #[must_use]
    pub const fn control(self) -> Color32 {
        self.control
    }

    /// Cor de bordas.
    #[must_use]
    pub const fn border(self) -> Color32 {
        self.border
    }

    /// Cor de texto primário.
    #[must_use]
    pub const fn text(self) -> Color32 {
        self.text
    }

    /// Cor de texto secundário/atenuado.
    #[must_use]
    pub const fn muted(self) -> Color32 {
        self.muted
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

    /// Cor de destaque (botão primário, pílula ativa).
    #[must_use]
    pub const fn accent(self) -> Color32 {
        self.accent
    }

    /// Cor do texto sobre o destaque.
    #[must_use]
    pub const fn on_accent(self) -> Color32 {
        self.on_accent
    }
}

#[cfg(test)]
mod tests;
