//! Adapter de estado da UI e bootstrap da janela eframe.

use crate::commands::AppCommands;
use application::ports::{DeviceListState, IsoSelection, UiCommands, UiState, WriteState};
use application::use_cases::ListDevices;
use infrastructure::linux::SysfsDiskService;
use infrastructure::screenshot::PngScreenshotWriter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use ui::theme::ThemePreference;
use ui::{DemoScenario, NurApp};

/// Estado da UI alimentado pela task de polling do sysfs e pela gravação.
pub(crate) struct LiveUiState {
    devices: Arc<RwLock<DeviceListState>>,
    write: Arc<RwLock<WriteState>>,
    iso: Arc<RwLock<Option<IsoSelection>>>,
}

impl LiveUiState {
    /// Cria o estado a partir dos locks compartilhados com os comandos.
    pub(crate) fn new(
        devices: Arc<RwLock<DeviceListState>>,
        write: Arc<RwLock<WriteState>>,
        iso: Arc<RwLock<Option<IsoSelection>>>,
    ) -> Self {
        Self {
            devices,
            write,
            iso,
        }
    }

    /// Spawna a task que enumera os pendrives a cada 1,5s e repinta a UI.
    pub(crate) fn spawn_polling(
        runtime: &tokio::runtime::Handle,
        ctx: eframe::egui::Context,
        devices: Arc<RwLock<DeviceListState>>,
    ) {
        runtime.spawn(async move {
            let uc = ListDevices::new(Arc::new(SysfsDiskService::new()));
            loop {
                let next = match uc.execute().await {
                    Ok(views) => DeviceListState::Ready(views),
                    Err(e) => {
                        DeviceListState::Error(format!("falha ao detectar dispositivos: {e}"))
                    }
                };
                if let Ok(mut guard) = devices.write() {
                    *guard = next;
                }
                ctx.request_repaint();
                tokio::time::sleep(Duration::from_secs_f32(1.5)).await;
            }
        });
    }
}

impl UiState for LiveUiState {
    fn device_list(&self) -> DeviceListState {
        self.devices
            .read()
            .map_or(DeviceListState::Loading, |guard| guard.clone())
    }

    fn write_state(&self) -> WriteState {
        self.write
            .read()
            .map_or(WriteState::Idle, |guard| guard.clone())
    }

    fn selected_iso(&self) -> Option<IsoSelection> {
        self.iso.read().ok().and_then(|guard| guard.clone())
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
                let ctx = cc.egui_ctx.clone();
                let devices = Arc::new(RwLock::new(DeviceListState::Loading));
                let write = Arc::new(RwLock::new(WriteState::Idle));
                let iso = Arc::new(RwLock::new(None));
                LiveUiState::spawn_polling(&runtime, ctx.clone(), Arc::clone(&devices));
                let state = Arc::new(LiveUiState::new(
                    devices,
                    Arc::clone(&write),
                    Arc::clone(&iso),
                ));
                let commands = Arc::new(AppCommands::new(runtime.clone(), ctx, write, iso));
                Ok(Box::new(Self::build_app(state, commands)))
            }),
        )
        .map_err(|e| anyhow::anyhow!("falha ao iniciar a janela: {e}"))
    }

    // Monta o app injetando estado, comandos e a config lida do ambiente.
    fn build_app(state: Arc<dyn UiState>, commands: Arc<dyn UiCommands>) -> NurApp {
        let screenshots = Arc::new(PngScreenshotWriter::new());
        NurApp::new(state, commands, screenshots)
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
