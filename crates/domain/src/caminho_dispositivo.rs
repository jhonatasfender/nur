//! Caminho de um dispositivo de bloco (ex.: `/dev/sdb`).

/// Caminho de dispositivo de bloco no sistema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaminhoDispositivo(String);

impl CaminhoDispositivo {
    /// Cria a partir de um caminho já validado pelo adapter de SO.
    #[must_use]
    pub fn new(caminho: String) -> Self {
        Self(caminho)
    }

    /// Retorna o caminho como string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
