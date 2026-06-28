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

/// Estado da lista de dispositivos exibido pela UI.
#[derive(Debug, Clone)]
pub enum DeviceListState {
    /// Detecção em andamento.
    Loading,
    /// Lista pronta (pode estar vazia).
    Ready(Vec<DeviceView>),
    /// Falha ao detectar (mensagem para o usuário).
    Error(String),
}

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Estado atual da lista de dispositivos.
    fn device_list(&self) -> DeviceListState;

    /// Estado atual da gravação (padrão: ociosa).
    fn write_state(&self) -> crate::ports::WriteState {
        crate::ports::WriteState::Idle
    }

    /// ISO selecionada pelo usuário, se houver (padrão: nenhuma).
    fn selected_iso(&self) -> Option<crate::ports::IsoSelection> {
        None
    }
}

#[cfg(test)]
mod tests;
