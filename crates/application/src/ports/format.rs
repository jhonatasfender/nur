//! Opções de formatação escolhidas pelo usuário.

use domain::{FilesystemKind, PartitionScheme, VolumeLabel};

/// Como formatar o dispositivo.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    scheme: PartitionScheme,
    filesystem: FilesystemKind,
    label: VolumeLabel,
    quick: bool,
}

impl FormatOptions {
    /// Cria as opções.
    #[must_use]
    pub fn new(
        scheme: PartitionScheme,
        filesystem: FilesystemKind,
        label: VolumeLabel,
        quick: bool,
    ) -> Self {
        Self {
            scheme,
            filesystem,
            label,
            quick,
        }
    }

    /// Esquema de partição.
    #[must_use]
    pub fn scheme(&self) -> PartitionScheme {
        self.scheme
    }

    /// Filesystem a criar.
    #[must_use]
    pub fn filesystem(&self) -> FilesystemKind {
        self.filesystem
    }

    /// Rótulo do volume.
    #[must_use]
    pub fn label(&self) -> &VolumeLabel {
        &self.label
    }

    /// Formatação rápida (sem zerar o disco).
    #[must_use]
    pub fn quick(&self) -> bool {
        self.quick
    }
}
