//! Tipos e portas auxiliares da gravação (estado, progresso, cancelamento).

use domain::{ByteSize, DevicePath, IsoKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Fase em andamento de uma gravação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePhase {
    /// Abrindo/preparando o dispositivo.
    Preparing,
    /// Escrevendo a imagem.
    Writing,
    /// Relendo e comparando com a imagem.
    Verifying,
}

/// Progresso instantâneo reportado pelo gravador.
#[derive(Debug, Clone, Copy)]
pub struct WriteProgress {
    phase: WritePhase,
    done: u64,
    total: u64,
}

impl WriteProgress {
    /// Cria um progresso.
    #[must_use]
    pub fn new(phase: WritePhase, done: u64, total: u64) -> Self {
        Self { phase, done, total }
    }

    /// Fase atual.
    #[must_use]
    pub fn phase(&self) -> WritePhase {
        self.phase
    }

    /// Bytes processados na fase.
    #[must_use]
    pub fn done(&self) -> u64 {
        self.done
    }

    /// Total de bytes da fase.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.total
    }
}

/// Estado da gravação lido pela UI.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteState {
    /// Nada em andamento.
    Idle,
    /// Preparando o dispositivo.
    Preparing,
    /// Gravando (determinado).
    Writing {
        /// Bytes gravados.
        done: u64,
        /// Total de bytes.
        total: u64,
    },
    /// Verificando (determinado).
    Verifying {
        /// Bytes verificados.
        done: u64,
        /// Total de bytes.
        total: u64,
    },
    /// Concluído com sucesso.
    Done,
    /// Falhou (mensagem para o usuário).
    Failed(String),
    /// Cancelado pelo usuário.
    Cancelled,
}

/// Pedido de gravação: qual ISO em qual dispositivo.
#[derive(Debug, Clone)]
pub struct WriteRequest {
    iso_path: PathBuf,
    device: DevicePath,
}

impl WriteRequest {
    /// Cria o pedido.
    #[must_use]
    pub fn new(iso_path: PathBuf, device: DevicePath) -> Self {
        Self { iso_path, device }
    }

    /// Caminho da imagem ISO.
    #[must_use]
    pub fn iso_path(&self) -> &Path {
        &self.iso_path
    }

    /// Dispositivo de destino.
    #[must_use]
    pub fn device(&self) -> &DevicePath {
        &self.device
    }
}

/// ISO escolhida pelo usuário (projeção para a UI).
#[derive(Debug, Clone)]
pub struct IsoSelection {
    name: String,
    size: ByteSize,
    kind: IsoKind,
}

impl IsoSelection {
    /// Cria a seleção.
    #[must_use]
    pub fn new(name: String, size: ByteSize, kind: IsoKind) -> Self {
        Self { name, size, kind }
    }

    /// Nome do arquivo (sem diretório).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Tamanho da ISO.
    #[must_use]
    pub fn size(&self) -> ByteSize {
        self.size
    }

    /// Classificação da ISO.
    #[must_use]
    pub fn kind(&self) -> IsoKind {
        self.kind
    }
}

/// Sinalizador de cancelamento compartilhável entre threads.
#[derive(Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    /// Cria um sinalizador não-acionado.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Aciona o cancelamento (latched).
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Indica se o cancelamento foi solicitado.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Destino do progresso reportado pelo gravador.
pub trait ProgressSink: Send + Sync {
    /// Recebe um progresso instantâneo.
    fn report(&self, progress: WriteProgress);
}

#[cfg(test)]
mod tests;
