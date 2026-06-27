//! Porta de estado da UI: o que a tela lê para se desenhar.

/// Projeção de um dispositivo para exibição na UI (sem tipos de domínio).
#[derive(Debug, Clone)]
pub struct DeviceView {
    path: String,
    description: String,
}

impl DeviceView {
    /// Cria uma projeção com caminho e descrição.
    #[must_use]
    pub fn new(path: String, description: String) -> Self {
        Self { path, description }
    }

    /// Caminho do dispositivo (ex.: `/dev/sdb`).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Descrição legível (modelo — tamanho (caminho)).
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Lista de dispositivos para popular o seletor.
    fn devices(&self) -> Vec<DeviceView>;
}
