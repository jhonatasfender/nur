//! Rótulo de volume com validação (limite FAT de 11 caracteres).

/// Erro de rótulo de volume inválido.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RotuloInvalido {
    /// O rótulo estava vazio.
    #[error("rótulo vazio")]
    Vazio,
    /// O rótulo excede 11 caracteres.
    #[error("rótulo excede 11 caracteres")]
    MuitoLongo,
}

/// Rótulo de volume válido (1–11 caracteres).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotuloVolume(String);

impl RotuloVolume {
    /// Valida e cria um rótulo. Erros: vazio ou acima de 11 caracteres.
    ///
    /// # Errors
    /// Retorna [`RotuloInvalido`] quando a string não respeita o limite.
    pub fn parse(texto: &str) -> Result<Self, RotuloInvalido> {
        if texto.is_empty() {
            return Err(RotuloInvalido::Vazio);
        }
        if texto.chars().count() > 11 {
            return Err(RotuloInvalido::MuitoLongo);
        }
        Ok(Self(texto.to_owned()))
    }

    /// Retorna o rótulo como string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
