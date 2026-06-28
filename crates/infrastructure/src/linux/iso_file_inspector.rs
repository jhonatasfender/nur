//! Inspeção de ISO lendo os primeiros setores (detecção isohybrid). Rust puro.

use application::errors::IsoError;
use application::ports::IsoInspector;
use domain::IsoKind;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Classifica ISOs lendo o MBR (0x1BE/0x55AA) e o PVD (`CD001` em 0x8001).
pub struct IsoFileInspector;

impl IsoFileInspector {
    /// Cria o inspetor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Regra: assinatura 0x55AA + ≥1 partição não-vazia em 0x1BE → isohybrid.
    fn classify_bytes(mbr: &[u8; 512], _cd001: bool) -> IsoKind {
        let signature = mbr[510] == 0x55 && mbr[511] == 0xAA;
        let has_partition = (0..4).any(|i| {
            let start = 0x1BE + i * 16;
            mbr[start..start + 16].iter().any(|&b| b != 0)
        });
        if signature && has_partition {
            IsoKind::Isohybrid
        } else {
            IsoKind::Unsupported
        }
    }

    fn read_and_classify(path: &Path) -> Result<IsoKind, IsoError> {
        let mut file = std::fs::File::open(path).map_err(|e| IsoError::Io(e.to_string()))?;
        let mut mbr = [0u8; 512];
        file.read_exact(&mut mbr)
            .map_err(|e| IsoError::Io(e.to_string()))?;
        // CD001 em 0x8001 (setor 16). Best-effort: ausência não impede a regra.
        let mut cd = [0u8; 5];
        let cd001 = file
            .seek(SeekFrom::Start(0x8001))
            .and_then(|_| file.read_exact(&mut cd).map(|()| &cd == b"CD001"))
            .unwrap_or(false);
        Ok(Self::classify_bytes(&mbr, cd001))
    }
}

impl Default for IsoFileInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl IsoInspector for IsoFileInspector {
    async fn classify(&self, iso: &Path) -> Result<IsoKind, IsoError> {
        let path = iso.to_path_buf();
        tokio::task::spawn_blocking(move || Self::read_and_classify(&path))
            .await
            .map_err(|e| IsoError::Io(e.to_string()))?
    }
}

#[cfg(test)]
mod tests;
