//! Formata via udisks2: tabela de partição → 1 partição → mkfs do filesystem.
//!
//! Casca fina sobre o udisks2 (zbus blocking em `spawn_blocking`). O mapeamento
//! das opções para as strings do udisks é puro e testável; o fluxo D-Bus é
//! validado por loopback. A GUI nunca roda como root (o polkit autoriza).

use application::errors::FormatError;
use application::ports::{DeviceFormatter, FormatOptions};
use domain::{DevicePath, FilesystemKind, PartitionScheme};
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, Value};

/// Formata um dispositivo usando o udisks2.
pub struct Udisks2Formatter;

impl Udisks2Formatter {
    /// Cria o formatador.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Esquema de partição → tipo de tabela do udisks.
    fn udisks_table(scheme: PartitionScheme) -> &'static str {
        match scheme {
            PartitionScheme::Gpt => "gpt",
            PartitionScheme::Mbr => "dos",
        }
    }

    // Filesystem → tipo aceito pelo udisks `Block.Format`.
    fn udisks_fs(fs: FilesystemKind) -> &'static str {
        match fs {
            FilesystemKind::Fat32 => "vfat",
            FilesystemKind::Ntfs => "ntfs",
            FilesystemKind::ExFat => "exfat",
            FilesystemKind::Ext4 => "ext4",
        }
    }

    // Filesystem → nome humano para mensagens de erro.
    fn human_fs(fs: FilesystemKind) -> &'static str {
        match fs {
            FilesystemKind::Fat32 => "FAT32",
            FilesystemKind::Ntfs => "NTFS",
            FilesystemKind::ExFat => "exFAT",
            FilesystemKind::Ext4 => "ext4",
        }
    }

    // Traduz a mensagem de erro do D-Bus em uma variante de FormatError.
    fn classify_err(message: &str, fs_human: &str) -> FormatError {
        let lower = message.to_lowercase();
        if lower.contains("notauthorized") || lower.contains("not authorized") {
            FormatError::Unauthorized
        } else if lower.contains("busy") || lower.contains("mounted") || lower.contains("in use") {
            FormatError::DeviceBusy
        } else if !fs_human.is_empty()
            && (lower.contains("not found")
                || lower.contains("failed to execute")
                || lower.contains("no such file"))
        {
            // Só o passo de mkfs (com `fs_human` preenchido) pode faltar ferramenta;
            // nas chamadas de tabela/partição isso seria um falso-positivo.
            FormatError::ToolMissing(fs_human.to_owned())
        } else {
            FormatError::Backend(message.to_owned())
        }
    }

    // Cria a tabela de partição vazia no device.
    fn format_table(conn: &Connection, dev_path: &str, table: &str) -> Result<(), FormatError> {
        let options: HashMap<&str, Value> = HashMap::new();
        conn.call_method(
            Some("org.freedesktop.UDisks2"),
            dev_path,
            Some("org.freedesktop.UDisks2.Block"),
            "Format",
            &(table, options),
        )
        .map(drop)
        .map_err(|e| Self::classify_err(&e.to_string(), ""))
    }

    // Cria uma partição cobrindo o disco; devolve o object path da partição.
    fn create_partition(conn: &Connection, dev_path: &str) -> Result<String, FormatError> {
        let options: HashMap<&str, Value> = HashMap::new();
        let reply = conn
            .call_method(
                Some("org.freedesktop.UDisks2"),
                dev_path,
                Some("org.freedesktop.UDisks2.PartitionTable"),
                "CreatePartition",
                &(0u64, 0u64, "", "", options),
            )
            .map_err(|e| Self::classify_err(&e.to_string(), ""))?;
        let part: OwnedObjectPath = reply
            .body()
            .deserialize()
            .map_err(|e| FormatError::Backend(e.to_string()))?;
        Ok(part.as_str().to_owned())
    }

    // Formata a partição com o filesystem e o rótulo escolhidos.
    fn format_partition(
        conn: &Connection,
        part_path: &str,
        fs: &str,
        label: &str,
        quick: bool,
        fs_human: &str,
    ) -> Result<(), FormatError> {
        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("label", Value::from(label));
        if !quick {
            options.insert("erase", Value::from("zero"));
        }
        conn.call_method(
            Some("org.freedesktop.UDisks2"),
            part_path,
            Some("org.freedesktop.UDisks2.Block"),
            "Format",
            &(fs, options),
        )
        .map(drop)
        .map_err(|e| Self::classify_err(&e.to_string(), fs_human))
    }

    // Fluxo bloqueante completo.
    fn run(
        name: &str,
        table: &str,
        fs: &str,
        label: &str,
        quick: bool,
        fs_human: &str,
    ) -> Result<(), FormatError> {
        let conn = Connection::system().map_err(|e| FormatError::Backend(e.to_string()))?;
        let dev_path = format!("/org/freedesktop/UDisks2/block_devices/{name}");
        Self::format_table(&conn, &dev_path, table)?;
        let part_path = Self::create_partition(&conn, &dev_path)?;
        Self::format_partition(&conn, &part_path, fs, label, quick, fs_human)
    }
}

impl Default for Udisks2Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DeviceFormatter for Udisks2Formatter {
    async fn format(
        &self,
        device: &DevicePath,
        options: &FormatOptions,
    ) -> Result<(), FormatError> {
        let name = device.as_str().trim_start_matches("/dev/").to_owned();
        let table = Self::udisks_table(options.scheme()).to_owned();
        let fs = Self::udisks_fs(options.filesystem()).to_owned();
        let fs_human = Self::human_fs(options.filesystem()).to_owned();
        let label = options.label().as_str().to_owned();
        let quick = options.quick();
        tokio::task::spawn_blocking(move || Self::run(&name, &table, &fs, &label, quick, &fs_human))
            .await
            .map_err(|e| FormatError::Backend(e.to_string()))?
    }
}

#[cfg(test)]
mod tests;
