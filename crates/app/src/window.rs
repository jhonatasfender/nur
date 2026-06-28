//! Adapter de estado da UI e bootstrap da janela eframe.

use application::ports::{DeviceListState, UiState};
use application::use_cases::ListDevices;
use infrastructure::linux::SysfsDiskService;
use infrastructure::screenshot::PngScreenshotWriter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use ui::theme::ThemePreference;
use ui::{DemoScenario, NurApp};

/// Estado da UI alimentado por uma task de polling do udisks2 em background.
pub(crate) struct LiveUiState {
    shared: Arc<RwLock<DeviceListState>>,
}

impl LiveUiState {
    /// Cria o estado (inicia em `Loading`) e spawna a task que faz polling do
    /// udisks2 a cada 1,5s, atualizando o estado e repintando a UI.
    pub(crate) fn spawn(runtime: &tokio::runtime::Handle, ctx: eframe::egui::Context) -> Self {
        let shared = Arc::new(RwLock::new(DeviceListState::Loading));
        let writer = Arc::clone(&shared);
        runtime.spawn(async move {
            let uc = ListDevices::new(Arc::new(SysfsDiskService::new()));
            loop {
                let next = match uc.execute().await {
                    Ok(views) => DeviceListState::Ready(views),
                    Err(e) => {
                        DeviceListState::Error(format!("falha ao detectar dispositivos: {e}"))
                    }
                };
                if let Ok(mut guard) = writer.write() {
                    *guard = next;
                }
                ctx.request_repaint();
                tokio::time::sleep(Duration::from_secs_f32(1.5)).await;
            }
        });
        Self { shared }
    }
}

impl UiState for LiveUiState {
    fn device_list(&self) -> DeviceListState {
        self.shared
            .read()
            .map_or(DeviceListState::Loading, |guard| guard.clone())
    }
}

/// Bootstrap da janela do Nur: monta o app (com adapters/config) e abre o eframe.
pub(crate) struct Window;

impl Window {
    /// Abre a janela do Nur. Bloqueia até o usuário fechar.
    ///
    /// # Errors
    /// Retorna erro se o eframe falhar ao iniciar.
    pub(crate) fn open(runtime: tokio::runtime::Handle) -> anyhow::Result<()> {
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
            Box::new(move |cc| {
                // A ponte tokio→egui nasce aqui (precisa do egui_ctx).
                let state = LiveUiState::spawn(&runtime, cc.egui_ctx.clone());
                Ok(Box::new(Self::build_app(Arc::new(state))))
            }),
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
