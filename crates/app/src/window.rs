//! Adapter de estado da UI e bootstrap da janela eframe.

use application::ports::{DispositivoView, UiState};
use application::use_cases::ListarDispositivos;
use infrastructure::stub::DiskServiceStub;
use std::sync::Arc;

/// Estado da UI alimentado pelo caso de uso (sobre o stub nesta fase).
pub(crate) struct UiStateAoVivo {
    dispositivos: Vec<DispositivoView>,
}

impl UiStateAoVivo {
    /// Monta o estado executando a listagem uma vez.
    ///
    /// # Errors
    /// Propaga falha do caso de uso.
    pub(crate) fn montar() -> anyhow::Result<Self> {
        let uc = ListarDispositivos::new(Arc::new(DiskServiceStub::new()));
        let dispositivos = uc.executar()?;
        Ok(Self { dispositivos })
    }
}

impl UiState for UiStateAoVivo {
    fn dispositivos(&self) -> Vec<DispositivoView> {
        self.dispositivos.clone()
    }
}

/// Abre a janela do Nur. Bloqueia até o usuário fechar.
///
/// # Errors
/// Retorna erro se o eframe falhar ao iniciar.
pub(crate) fn abrir(estado: Arc<dyn UiState>) -> anyhow::Result<()> {
    let opcoes = eframe::NativeOptions::default();
    eframe::run_native(
        "Nur",
        opcoes,
        Box::new(|_cc| Ok(Box::new(ui::NurApp::new(estado)))),
    )
    .map_err(|e| anyhow::anyhow!("falha ao iniciar a janela: {e}"))
}
