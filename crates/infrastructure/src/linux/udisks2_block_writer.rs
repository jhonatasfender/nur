//! Gravador que abre o dispositivo via udisks2/polkit e grava/verifica.
//!
//! `Block.OpenDevice` devolve um fd já autorizado pelo polkit (a GUI nunca roda
//! como root). A chamada zbus é blocking e isolada em `spawn_blocking` — é uma
//! única chamada por abertura, então a lentidão da enumeração (Fase 3) não vale.
//! A lógica de cópia/verificação testável mora no [`RawCopier`].

use crate::linux::RawCopier;
use application::errors::WriteError;
use application::ports::{
    BootableWriter, CancelFlag, ProgressSink, WritePhase, WriteProgress, WriteRequest,
};
use std::collections::HashMap;
use std::sync::Arc;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedFd, Value};

// O_EXCL | O_SYNC | O_CLOEXEC — exclusivo (falha se montado), síncrono, close-on-exec.
const FLAGS: i32 = 0x80 | 0x0010_1000 | 0x0008_0000;

/// Grava a ISO no dispositivo usando um fd autorizado pelo udisks2/polkit.
pub struct Udisks2BlockWriter;

impl Udisks2BlockWriter {
    /// Cria o gravador.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Tamanho do dispositivo em bytes, lido do sysfs (setores × 512).
    fn device_size(name: &str) -> Result<u64, WriteError> {
        let raw = std::fs::read_to_string(format!("/sys/block/{name}/size"))
            .map_err(|e| WriteError::Io(e.to_string()))?;
        let sectors: u64 = raw
            .trim()
            .parse()
            .map_err(|_| WriteError::Io("tamanho do dispositivo inválido".to_owned()))?;
        Ok(sectors.saturating_mul(512))
    }

    // Traduz erros do D-Bus/polkit em variantes de WriteError.
    fn map_dbus_err(e: &zbus::Error) -> WriteError {
        let msg = e.to_string();
        let lower = msg.to_lowercase();
        if lower.contains("notauthorized") || lower.contains("not authorized") {
            WriteError::Unauthorized
        } else if lower.contains("busy") || lower.contains("mounted") || lower.contains("in use") {
            WriteError::DeviceBusy
        } else {
            WriteError::Io(msg)
        }
    }

    // Abre o dispositivo via udisks2 e devolve um File a partir do fd autorizado.
    fn open_device(name: &str, mode: &str) -> Result<std::fs::File, WriteError> {
        let conn = Connection::system().map_err(|e| Self::map_dbus_err(&e))?;
        let path = format!("/org/freedesktop/UDisks2/block_devices/{name}");
        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("flags", Value::from(FLAGS));
        let reply = conn
            .call_method(
                Some("org.freedesktop.UDisks2"),
                path.as_str(),
                Some("org.freedesktop.UDisks2.Block"),
                "OpenDevice",
                &(mode, options),
            )
            .map_err(|e| Self::map_dbus_err(&e))?;
        let fd: OwnedFd = reply
            .body()
            .deserialize()
            .map_err(|e| Self::map_dbus_err(&e))?;
        // From<OwnedFd> para File é seguro (sem `unsafe`).
        Ok(std::fs::File::from(std::os::fd::OwnedFd::from(fd)))
    }

    // Fluxo bloqueante completo: validar → abrir → gravar → sync → verificar.
    fn run(
        request: &WriteRequest,
        sink: &Arc<dyn ProgressSink>,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError> {
        let name = request.device().as_str().trim_start_matches("/dev/");
        let iso_len = std::fs::metadata(request.iso_path())
            .map_err(|e| WriteError::Io(e.to_string()))?
            .len();
        if iso_len > Self::device_size(name)? {
            return Err(WriteError::DeviceTooSmall);
        }
        sink.report(WriteProgress::new(WritePhase::Preparing, 0, 0));
        {
            let mut dev = Self::open_device(name, "rw")?;
            let mut iso = std::fs::File::open(request.iso_path())
                .map_err(|e| WriteError::Io(e.to_string()))?;
            RawCopier::copy(&mut iso, &mut dev, iso_len, sink.as_ref(), cancel)?;
            dev.sync_all().map_err(|e| WriteError::Io(e.to_string()))?;
        }
        // Reabre para a leitura de verificação (evita o cache do fd de escrita).
        let mut dev_read = Self::open_device(name, "r")?;
        let mut iso2 =
            std::fs::File::open(request.iso_path()).map_err(|e| WriteError::Io(e.to_string()))?;
        RawCopier::verify(&mut dev_read, &mut iso2, iso_len, sink.as_ref(), cancel)
    }
}

impl Default for Udisks2BlockWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BootableWriter for Udisks2BlockWriter {
    async fn write(
        &self,
        request: &WriteRequest,
        sink: Arc<dyn ProgressSink>,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError> {
        let request = request.clone();
        let cancel = cancel.clone();
        tokio::task::spawn_blocking(move || Self::run(&request, &sink, &cancel))
            .await
            .map_err(|e| WriteError::Io(e.to_string()))?
    }
}
