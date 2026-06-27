//! Caminho de um dispositivo de bloco (ex.: `/dev/sdb`).

/// Caminho de dispositivo de bloco no sistema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePath(String);

impl DevicePath {
    /// Cria a partir de um caminho já validado pelo adapter de SO.
    #[must_use]
    pub fn new(path: String) -> Self {
        Self(path)
    }

    /// Retorna o caminho como string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
