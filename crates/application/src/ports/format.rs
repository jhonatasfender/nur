//! Opções de formatação escolhidas pelo usuário.

use domain::{PartitionScheme, VolumeLabel};

/// Como formatar o dispositivo (sempre FAT32 neste incremento).
#[derive(Debug, Clone)]
pub struct FormatOptions {
    scheme: PartitionScheme,
    label: VolumeLabel,
    quick: bool,
}

impl FormatOptions {
    /// Cria as opções.
    #[must_use]
    pub fn new(scheme: PartitionScheme, label: VolumeLabel, quick: bool) -> Self {
        Self {
            scheme,
            label,
            quick,
        }
    }

    /// Esquema de partição.
    #[must_use]
    pub fn scheme(&self) -> PartitionScheme {
        self.scheme
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
