//! Adapter de estado da UI e bootstrap da janela eframe.

use application::ports::{DeviceView, UiState};
use application::use_cases::ListDevices;
use infrastructure::screenshot::PngScreenshotWriter;
use infrastructure::stub::DiskServiceStub;
use std::path::PathBuf;
use std::sync::Arc;
use ui::theme::ThemePreference;
use ui::{DemoScenario, NurApp};

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

/// Bootstrap da janela do Nur: monta o app (com adapters/config) e abre o eframe.
pub(crate) struct Window;

impl Window {
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
            Box::new(|_cc| Ok(Box::new(Self::build_app(state)))),
        )
        .map_err(|e| anyhow::anyhow!("falha ao iniciar a janela: {e}"))
    }

    // Monta o app injetando o gravador real e a config lida do ambiente.
    fn build_app(state: Arc<dyn UiState>) -> NurApp {
        let screenshots = Arc::new(PngScreenshotWriter::new());
        NurApp::new(state, screenshots)
            .with_theme(Self::theme_from_env())
            .with_capture_path(Self::capture_path_from_env())
            .with_demo(Self::demo_from_env())
    }

    // Tema inicial: `NUR_THEME=light` força o tema claro; senão, escuro.
    fn theme_from_env() -> ThemePreference {
        if std::env::var("NUR_THEME").as_deref() == Ok("light") {
            ThemePreference::Light
        } else {
            ThemePreference::Dark
        }
    }

    // Destino de captura automática (modo headless), via `NUR_CAPTURE`.
    fn capture_path_from_env() -> Option<PathBuf> {
        std::env::var_os("NUR_CAPTURE").map(PathBuf::from)
    }

    // Cenário de demonstração para validação visual, via `NUR_DEMO`.
    fn demo_from_env() -> Option<DemoScenario> {
        match std::env::var("NUR_DEMO").as_deref() {
            Ok("ready") => Some(DemoScenario::Ready),
            Ok("modal") => Some(DemoScenario::Modal),
            Ok("running") => Some(DemoScenario::Running),
            Ok("format") => Some(DemoScenario::Format),
            _ => None,
        }
    }
}
