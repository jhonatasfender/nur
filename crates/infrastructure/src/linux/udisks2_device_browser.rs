//! Abre o pendrive no gerenciador de arquivos: monta via udisks2 e `xdg-open`.
//!
//! Read-only e fora da aplicação — só pedimos ao SO para montar (como o desktop
//! faz ao plugar) e abrir a pasta no gerenciador nativo. A escolha do ponto de
//! montagem (parsing de `/proc/mounts`) mora no [`MountTable`], testável.

use crate::linux::MountTable;
use application::errors::BrowseError;
use application::ports::DeviceBrowser;
use domain::DevicePath;
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::zvariant::Value;

/// Abre o conteúdo de um pendrive no gerenciador de arquivos do SO (Linux).
pub struct Udisks2DeviceBrowser;

impl Udisks2DeviceBrowser {
    /// Cria o navegador.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Fluxo bloqueante: achar montagem existente ou montar, depois abrir.
    fn run(name: &str) -> Result<(), BrowseError> {
        let mount_point = match Self::mounted_path(name) {
            Some(path) => path,
            None => Self::mount_first_partition(name)?,
        };
        Self::launch(&mount_point)
    }

    // Ponto de montagem de uma partição já montada deste device, se houver.
    fn mounted_path(name: &str) -> Option<String> {
        let contents = std::fs::read_to_string("/proc/mounts").ok()?;
        MountTable::mount_point_for(&contents, name)
    }

    // Nomes das partições do device (ex.: `sdb1`, `sdb2`), em ordem.
    fn partitions(name: &str) -> Vec<String> {
        let mut parts: Vec<String> = std::fs::read_dir(format!("/sys/block/{name}"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|n| n.starts_with(name) && n.len() > name.len())
            .collect();
        parts.sort();
        parts
    }

    // Monta a 1ª partição com filesystem e devolve o ponto de montagem.
    fn mount_first_partition(name: &str) -> Result<String, BrowseError> {
        let parts = Self::partitions(name);
        if parts.is_empty() {
            return Err(BrowseError::NoFilesystem);
        }
        let mut last = BrowseError::NoFilesystem;
        for part in parts {
            match Self::mount_via_udisks(&part) {
                Ok(path) => return Ok(path),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    // Chama udisks2 `Filesystem.Mount` na partição e devolve o caminho montado.
    fn mount_via_udisks(part: &str) -> Result<String, BrowseError> {
        let conn = Connection::system().map_err(|e| BrowseError::Mount(e.to_string()))?;
        let path = format!("/org/freedesktop/UDisks2/block_devices/{part}");
        let options: HashMap<&str, Value> = HashMap::new();
        let reply = conn
            .call_method(
                Some("org.freedesktop.UDisks2"),
                path.as_str(),
                Some("org.freedesktop.UDisks2.Filesystem"),
                "Mount",
                &(options,),
            )
            .map_err(|e| BrowseError::Mount(e.to_string()))?;
        reply
            .body()
            .deserialize::<String>()
            .map_err(|e| BrowseError::Mount(e.to_string()))
    }

    // Lança o gerenciador de arquivos nativo na pasta (sem aguardar).
    fn launch(path: &str) -> Result<(), BrowseError> {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(drop)
            .map_err(|e| BrowseError::Launch(e.to_string()))
    }
}

impl Default for Udisks2DeviceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DeviceBrowser for Udisks2DeviceBrowser {
    async fn open(&self, device: &DevicePath) -> Result<(), BrowseError> {
        let name = device.as_str().trim_start_matches("/dev/").to_owned();
        tokio::task::spawn_blocking(move || Self::run(&name))
            .await
            .map_err(|e| BrowseError::Launch(e.to_string()))?
    }
}
