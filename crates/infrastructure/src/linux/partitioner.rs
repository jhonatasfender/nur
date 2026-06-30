//! Escreve a tabela de partição (GPT/MBR) com 1 partição FAT32 cobrindo o disco.
//! Rust puro (crates `gpt`/`mbrman`), sem ferramentas externas.

use domain::PartitionScheme;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::{self, Read, Seek, SeekFrom, Write};

const ALIGN: u64 = 1024 * 1024; // 1 MiB
const SECTOR: u64 = 512;

/// Particionador nativo.
pub(crate) struct Partitioner;

impl Partitioner {
    /// `(start, len)` em bytes para 1 partição cobrindo o disco (start a 1 MiB).
    pub(crate) fn single_partition(device_bytes: u64) -> (u64, u64) {
        if device_bytes <= ALIGN {
            return (ALIGN, 0);
        }
        let len = ((device_bytes - ALIGN) / SECTOR) * SECTOR;
        (ALIGN, len)
    }

    /// Escreve a tabela escolhida com 1 partição FAT32 em `[start, start+len)`.
    /// Devolve o `(start, len)` **real** da partição criada (o GPT pode realinhar).
    pub(crate) fn write_table<T: Read + Write + Seek + Debug>(
        dev: &mut T,
        scheme: PartitionScheme,
        start: u64,
        len: u64,
    ) -> io::Result<(u64, u64)> {
        match scheme {
            PartitionScheme::Gpt => Self::write_gpt(dev, len),
            PartitionScheme::Mbr => Self::write_mbr(dev, start, len),
        }
    }

    fn write_gpt<T: Read + Write + Seek + Debug>(dev: &mut T, len: u64) -> io::Result<(u64, u64)> {
        let lb = gpt::disk::LogicalBlockSize::Lb512;
        let total = dev.seek(SeekFrom::End(0))? / SECTOR;
        let pmbr = gpt::mbr::ProtectiveMBR::with_lb_size(
            u32::try_from(total.saturating_sub(1)).unwrap_or(0xFFFF_FFFF),
        );
        pmbr.overwrite_lba0(dev).map_err(Self::to_io)?;
        let mut disk = gpt::GptConfig::new()
            .writable(true)
            .logical_block_size(lb)
            .create_from_device(&mut *dev, None)
            .map_err(Self::to_io)?;
        disk.update_partitions(BTreeMap::new())
            .map_err(Self::to_io)?;
        disk.add_partition("WINUSB", len, gpt::partition_types::BASIC, 0, None)
            .map_err(Self::to_io)?;
        let part = disk
            .partitions()
            .get(&1)
            .ok_or_else(|| io::Error::other("partição GPT não criada"))?;
        let (first, last) = (part.first_lba, part.last_lba);
        disk.write().map_err(Self::to_io)?;
        Ok((first * SECTOR, (last - first + 1) * SECTOR))
    }

    // Escreve uma MBR (DOS) mínima com 1 partição FAT32 LBA (tipo 0x0C).
    // É só o setor 0 (512 bytes): entrada em 0x1BE + assinatura 0x55AA em 0x1FE.
    fn write_mbr<T: Write + Seek>(dev: &mut T, start: u64, len: u64) -> io::Result<(u64, u64)> {
        let start_lba = u32::try_from(start / SECTOR).unwrap_or(0);
        let sectors = u32::try_from(len / SECTOR).unwrap_or(0);
        let mut sector = [0u8; 512];
        let e = 0x1BE; // 1ª entrada de partição (16 bytes)
        sector[e] = 0x00; // não-ativa
        sector[e + 1..e + 4].copy_from_slice(&[0xFE, 0xFF, 0xFF]); // CHS first (placeholder LBA)
        sector[e + 4] = 0x0C; // tipo: FAT32 LBA
        sector[e + 5..e + 8].copy_from_slice(&[0xFE, 0xFF, 0xFF]); // CHS last
        sector[e + 8..e + 12].copy_from_slice(&start_lba.to_le_bytes());
        sector[e + 12..e + 16].copy_from_slice(&sectors.to_le_bytes());
        sector[510] = 0x55;
        sector[511] = 0xAA;
        dev.seek(SeekFrom::Start(0))?;
        dev.write_all(&sector)?;
        Ok((start, len))
    }

    fn to_io<E: std::fmt::Display>(e: E) -> io::Error {
        io::Error::other(e.to_string())
    }
}

#[cfg(test)]
mod tests;
