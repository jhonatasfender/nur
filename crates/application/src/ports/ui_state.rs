//! Porta de estado da UI: o que a tela lê para se desenhar.

/// Projeção de um dispositivo para exibição na UI (sem tipos de domínio).
#[derive(Debug, Clone)]
pub struct DispositivoView {
    /// Caminho do dispositivo (ex.: `/dev/sdb`).
    pub caminho: String,
    /// Descrição legível (modelo — tamanho (caminho)).
    pub descricao: String,
}

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Lista de dispositivos para popular o seletor.
    fn dispositivos(&self) -> Vec<DispositivoView>;
}
