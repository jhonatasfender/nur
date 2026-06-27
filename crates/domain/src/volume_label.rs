//! Rótulo de volume com validação (limite FAT de 11 caracteres).

/// Erro de rótulo de volume inválido.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InvalidLabel {
    /// O rótulo estava vazio.
    #[error("rótulo vazio")]
    Empty,
    /// O rótulo excede 11 caracteres.
    #[error("rótulo excede 11 caracteres")]
    TooLong,
}

/// Rótulo de volume válido (1–11 caracteres).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeLabel(String);

impl VolumeLabel {
    /// Valida e cria um rótulo. Erros: vazio ou acima de 11 caracteres.
    ///
    /// # Errors
    /// Retorna [`InvalidLabel`] quando a string não respeita o limite.
    pub fn parse(text: &str) -> Result<Self, InvalidLabel> {
        if text.is_empty() {
            return Err(InvalidLabel::Empty);
        }
        if text.chars().count() > 11 {
            return Err(InvalidLabel::TooLong);
        }
        Ok(Self(text.to_owned()))
    }

    /// Retorna o rótulo como string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
