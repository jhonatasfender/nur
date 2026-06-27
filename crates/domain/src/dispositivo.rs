//! Dispositivo de bloco detectado (pendrive) como agregado de domínio.

use crate::{ByteSize, CaminhoDispositivo};

/// Um dispositivo de armazenamento detectado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispositivo {
    caminho: CaminhoDispositivo,
    modelo: String,
    tamanho: ByteSize,
    removivel: bool,
}

impl Dispositivo {
    /// Cria um dispositivo a partir dos dados do adapter de SO.
    #[must_use]
    pub fn new(
        caminho: CaminhoDispositivo,
        modelo: String,
        tamanho: ByteSize,
        removivel: bool,
    ) -> Self {
        Self {
            caminho,
            modelo,
            tamanho,
            removivel,
        }
    }

    /// Caminho do dispositivo.
    #[must_use]
    pub fn caminho(&self) -> &CaminhoDispositivo {
        &self.caminho
    }

    /// Modelo do dispositivo.
    #[must_use]
    pub fn modelo(&self) -> &str {
        &self.modelo
    }

    /// Tamanho do dispositivo.
    #[must_use]
    pub fn tamanho(&self) -> ByteSize {
        self.tamanho
    }

    /// Indica se o dispositivo é removível.
    #[must_use]
    pub fn removivel(&self) -> bool {
        self.removivel
    }

    /// Descrição legível para a UI (modelo — tamanho (caminho)).
    #[must_use]
    pub fn descricao(&self) -> String {
        format!(
            "{} — {} ({})",
            self.modelo,
            self.tamanho.humanize(),
            self.caminho.as_str()
        )
    }
}

#[cfg(test)]
mod tests;
