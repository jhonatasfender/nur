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

    fn write_mbr<T: Read + Write + Seek + Debug>(
        dev: &mut T,
        start: u64,
        len: u64,
    ) -> io::Result<(u64, u64)> {
        let mut mbr = mbrman::MBR::new_from(dev, u32::try_from(SECTOR).unwrap_or(512), [0, 0, 0, 0])
            .map_err(Self::to_io)?;
        mbr[1] = mbrman::MBRPartitionEntry {
            boot: mbrman::BOOT_INACTIVE,
            first_chs: mbrman::CHS::empty(),
            sys: 0x0c,
            last_chs: mbrman::CHS::empty(),
            starting_lba: u32::try_from(start / SECTOR).unwrap_or(0),
            sectors: u32::try_from(len / SECTOR).unwrap_or(0),
        };
        mbr.write_into(dev).map_err(Self::to_io)?;
        Ok((start, len))
    }

    fn to_io<E: std::fmt::Display>(e: E) -> io::Error {
        io::Error::other(e.to_string())
    }
}

#[cfg(test)]
mod tests;
