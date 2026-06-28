//! Porta de comandos disparados pela UI (lado de escrita do app).

use domain::DevicePath;

/// Ações que a UI dispara; implementadas no composition root.
pub trait UiCommands: Send + Sync {
    /// Abre o diálogo nativo para escolher uma ISO e a inspeciona.
    fn pick_iso(&self);

    /// Inicia a gravação da ISO selecionada no dispositivo dado.
    fn start(&self, device: DevicePath);

    /// Solicita o cancelamento da gravação em andamento.
    fn cancel(&self);

    /// Abre o pendrive no gerenciador de arquivos do SO.
    fn open_device(&self, device: DevicePath);
}
