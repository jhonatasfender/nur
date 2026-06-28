//! Implementação dos comandos da UI (lado de escrita): seleção de ISO e gravação.
//!
//! Spawna tasks tokio que rodam o caso de uso e publicam o progresso nos locks
//! compartilhados com [`crate::window::LiveUiState`], repintando a UI.

use application::errors::WriteError;
use application::ports::{
    CancelFlag, DeviceBrowser, IsoInspector, IsoSelection, ProgressSink, UiCommands, WritePhase,
    WriteProgress, WriteRequest, WriteState,
};
use application::use_cases::CreateBootable;
use domain::{ByteSize, DevicePath, IsoKind};
use infrastructure::linux::{IsoFileInspector, Udisks2BlockWriter, Udisks2DeviceBrowser};
use infrastructure::picker::RfdIsoPicker;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Comandos da UI implementados no composition root.
pub(crate) struct AppCommands {
    runtime: tokio::runtime::Handle,
    ctx: eframe::egui::Context,
    write: Arc<RwLock<WriteState>>,
    iso: Arc<RwLock<Option<IsoSelection>>>,
    iso_path: Arc<RwLock<Option<PathBuf>>>,
    cancel: Arc<RwLock<CancelFlag>>,
    notice: Arc<RwLock<Option<String>>>,
}

impl AppCommands {
    /// Cria os comandos compartilhando os locks de estado com a UI.
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        ctx: eframe::egui::Context,
        write: Arc<RwLock<WriteState>>,
        iso: Arc<RwLock<Option<IsoSelection>>>,
        notice: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            runtime,
            ctx,
            write,
            iso,
            iso_path: Arc::new(RwLock::new(None)),
            cancel: Arc::new(RwLock::new(CancelFlag::new())),
            notice,
        }
    }
}

impl UiCommands for AppCommands {
    fn pick_iso(&self) {
        let ctx = self.ctx.clone();
        let iso = Arc::clone(&self.iso);
        let iso_path = Arc::clone(&self.iso_path);
        self.runtime.spawn(async move {
            let Some(path) = RfdIsoPicker::new().pick().await else {
                return;
            };
            let kind = IsoFileInspector::new()
                .classify(&path)
                .await
                .unwrap_or(IsoKind::Unsupported);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("imagem")
                .to_owned();
            let size = std::fs::metadata(&path).map_or(0, |m| m.len());
            if let Ok(mut guard) = iso.write() {
                *guard = Some(IsoSelection::new(name, ByteSize::from_bytes(size), kind));
            }
            if let Ok(mut guard) = iso_path.write() {
                *guard = Some(path);
            }
            ctx.request_repaint();
        });
    }

    fn start(&self, device: DevicePath) {
        let Some(path) = self.iso_path.read().ok().and_then(|g| g.clone()) else {
            return;
        };
        let flag = CancelFlag::new();
        if let Ok(mut guard) = self.cancel.write() {
            *guard = flag.clone();
        }
        let ctx = self.ctx.clone();
        let write = Arc::clone(&self.write);
        self.runtime.spawn(async move {
            let uc = CreateBootable::new(
                Arc::new(IsoFileInspector::new()),
                Arc::new(Udisks2BlockWriter::new()),
            );
            let sink: Arc<dyn ProgressSink> =
                Arc::new(AppProgressSink::new(Arc::clone(&write), ctx.clone()));
            let result = uc
                .execute(WriteRequest::new(path, device), sink, flag)
                .await;
            let next = match result {
                Ok(()) => WriteState::Done,
                Err(WriteError::Cancelled) => WriteState::Cancelled,
                Err(e) => WriteState::Failed(e.to_string()),
            };
            if let Ok(mut guard) = write.write() {
                *guard = next;
            }
            ctx.request_repaint();
        });
    }

    fn cancel(&self) {
        if let Ok(guard) = self.cancel.read() {
            guard.cancel();
        }
    }

    fn open_device(&self, device: DevicePath) {
        let ctx = self.ctx.clone();
        let notice = Arc::clone(&self.notice);
        self.runtime.spawn(async move {
            let result = Udisks2DeviceBrowser::new().open(&device).await;
            if let Ok(mut guard) = notice.write() {
                *guard = result.err().map(|e| e.to_string());
            }
            ctx.request_repaint();
        });
    }
}

// Traduz o progresso do gravador para o WriteState lido pela UI.
struct AppProgressSink {
    write: Arc<RwLock<WriteState>>,
    ctx: eframe::egui::Context,
}

impl AppProgressSink {
    fn new(write: Arc<RwLock<WriteState>>, ctx: eframe::egui::Context) -> Self {
        Self { write, ctx }
    }
}

impl ProgressSink for AppProgressSink {
    fn report(&self, progress: WriteProgress) {
        let next = match progress.phase() {
            WritePhase::Preparing => WriteState::Preparing,
            WritePhase::Writing => WriteState::Writing {
                done: progress.done(),
                total: progress.total(),
            },
            WritePhase::Verifying => WriteState::Verifying {
                done: progress.done(),
                total: progress.total(),
            },
        };
        if let Ok(mut guard) = self.write.write() {
            *guard = next;
        }
        self.ctx.request_repaint();
    }
}
