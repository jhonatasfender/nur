//! Formatador FAT32 nativo: abre o device via udisks/polkit (só o fd privilegiado)
//! e escreve tabela (GPT/MBR) + FAT32 em Rust puro — sem `mkfs`/`Block.Format`.

use super::fat32::Fat32;
use super::offset_volume::OffsetVolume;
use super::partitioner::Partitioner;
use application::errors::FormatError;
use application::ports::{DeviceFormatter, FormatOptions};
use domain::DevicePath;
use std::collections::HashMap;
use std::io::{Seek, SeekFrom, Write};
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedFd, Value};

// O_EXCL | O_SYNC | O_CLOEXEC — exclusivo (falha se montado), síncrono, close-on-exec.
const FLAGS: i32 = 0x80 | 0x0010_1000 | 0x0008_0000;
const ZERO_CHUNK: usize = 4 * 1024 * 1024;

/// Formata um dispositivo como GPT/MBR + FAT32, tudo em Rust.
pub struct NativeFatFormatter;

impl NativeFatFormatter {
    /// Cria o formatador.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Traduz erros do D-Bus/polkit em variantes de FormatError.
    fn map_dbus_err(e: &zbus::Error) -> FormatError {
        let lower = e.to_string().to_lowercase();
        if lower.contains("notauthorized") || lower.contains("not authorized") {
            FormatError::Unauthorized
        } else if lower.contains("busy") || lower.contains("mounted") || lower.contains("in use") {
            FormatError::DeviceBusy
        } else {
            FormatError::Backend(e.to_string())
        }
    }

    // Tamanho do dispositivo em bytes (sysfs, setores × 512).
    fn device_size(name: &str) -> Result<u64, FormatError> {
        let raw = std::fs::read_to_string(format!("/sys/block/{name}/size"))
            .map_err(|e| FormatError::Backend(e.to_string()))?;
        let sectors: u64 = raw
            .trim()
            .parse()
            .map_err(|_| FormatError::Backend("tamanho do dispositivo inválido".to_owned()))?;
        Ok(sectors.saturating_mul(512))
    }

    // Abre o device via udisks2 e devolve um File a partir do fd autorizado.
    fn open_device(name: &str) -> Result<std::fs::File, FormatError> {
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
                &("rw", options),
            )
            .map_err(|e| Self::map_dbus_err(&e))?;
        let fd: OwnedFd = reply
            .body()
            .deserialize()
            .map_err(|e| Self::map_dbus_err(&e))?;
        Ok(std::fs::File::from(std::os::fd::OwnedFd::from(fd)))
    }

    // Zera `[start, start+len)` (formatação completa).
    fn zero_region(file: &mut std::fs::File, start: u64, len: u64) -> Result<(), FormatError> {
        file.seek(SeekFrom::Start(start))
            .map_err(|e| FormatError::Backend(e.to_string()))?;
        let zeros = vec![0u8; ZERO_CHUNK];
        let mut written = 0u64;
        while written < len {
            let n = (len - written).min(ZERO_CHUNK as u64) as usize;
            file.write_all(&zeros[..n])
                .map_err(|e| FormatError::Backend(e.to_string()))?;
            written += n as u64;
        }
        Ok(())
    }

    // Fluxo bloqueante: validar tamanho → abrir → (zerar) → tabela → FAT32 → fsync.
    fn run(name: &str, options: &FormatOptions) -> Result<(), FormatError> {
        let (start, len) = Partitioner::single_partition(Self::device_size(name)?);
        if len == 0 {
            return Err(FormatError::Backend(
                "dispositivo muito pequeno para FAT32".to_owned(),
            ));
        }
        let mut file = Self::open_device(name)?;
        if !options.quick() {
            Self::zero_region(&mut file, start, len)?;
        }
        let (pstart, plen) = Partitioner::write_table(&mut file, options.scheme(), start, len)
            .map_err(|e| FormatError::Backend(e.to_string()))?;
        {
            let mut window = OffsetVolume::new(&mut file, pstart, plen);
            Fat32::format(&mut window, options.label())
                .map_err(|e| FormatError::Backend(e.to_string()))?;
        }
        file.sync_all()
            .map_err(|e| FormatError::Backend(e.to_string()))
    }
}

impl Default for NativeFatFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DeviceFormatter for NativeFatFormatter {
    async fn format(
        &self,
        device: &DevicePath,
        options: &FormatOptions,
    ) -> Result<(), FormatError> {
        let name = device.as_str().trim_start_matches("/dev/").to_owned();
        let options = options.clone();
        tokio::task::spawn_blocking(move || Self::run(&name, &options))
            .await
            .map_err(|e| FormatError::Backend(e.to_string()))?
    }
}
