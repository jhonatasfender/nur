//! Criação de FAT32 via crate `fatfs` (Rust puro, sem mkfs).

use domain::VolumeLabel;
use std::io::{Read, Seek, Write};

/// Formata um volume como FAT32.
pub(crate) struct Fat32;

impl Fat32 {
    /// Formata `volume` inteiro como FAT32 com o rótulo (padding a 11 bytes).
    pub(crate) fn format<T: Read + Write + Seek>(
        volume: &mut T,
        label: &VolumeLabel,
    ) -> std::io::Result<()> {
        let mut name = [b' '; 11];
        for (slot, byte) in name.iter_mut().zip(label.as_str().bytes()) {
            *slot = byte;
        }
        let options = fatfs::FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .volume_label(name);
        fatfs::format_volume(volume, options)
    }
}

#[cfg(test)]
mod tests;
