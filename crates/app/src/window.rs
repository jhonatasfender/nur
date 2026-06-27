//! Adapter de estado da UI e bootstrap da janela eframe.

use application::ports::{DeviceView, UiState};
use application::use_cases::ListDevices;
use infrastructure::stub::DiskServiceStub;
use std::sync::Arc;

/// Estado da UI alimentado pelo caso de uso (sobre o stub nesta fase).
pub(crate) struct LiveUiState {
    devices: Vec<DeviceView>,
}

impl LiveUiState {
    /// Monta o estado executando a listagem uma vez.
    ///
    /// # Errors
    /// Propaga falha do caso de uso.
    pub(crate) fn build() -> anyhow::Result<Self> {
        let uc = ListDevices::new(Arc::new(DiskServiceStub::new()));
        let devices = uc.execute()?;
        Ok(Self { devices })
    }
}

impl UiState for LiveUiState {
    fn devices(&self) -> Vec<DeviceView> {
        self.devices.clone()
    }
}

/// Abre a janela do Nur. Bloqueia até o usuário fechar.
///
/// # Errors
/// Retorna erro se o eframe falhar ao iniciar.
pub(crate) fn open(state: Arc<dyn UiState>) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([460.0, 720.0])
            .with_min_inner_size([440.0, 300.0])
            .with_resizable(false)
            .with_decorations(false) // sem barra de título nativa (todas as plataformas)
            .with_transparent(true) // cantos arredondados (fora do card fica transparente)
            .with_title("Nur"),
        ..Default::default()
    };
    eframe::run_native(
        "Nur",
        options,
        Box::new(|_cc| Ok(Box::new(ui::NurApp::new(state)))),
    )
    .map_err(|e| anyhow::anyhow!("falha ao iniciar a janela: {e}"))
}
