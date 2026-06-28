//! Value objects para a formatação: esquema de partição e filesystem.

/// Esquema da tabela de partição.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionScheme {
    /// GUID Partition Table.
    Gpt,
    /// Master Boot Record (DOS).
    Mbr,
}

/// Tipo de filesystem a criar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemKind {
    /// FAT32.
    Fat32,
    /// NTFS.
    Ntfs,
    /// exFAT.
    ExFat,
    /// ext4.
    Ext4,
}

#[cfg(test)]
mod tests;
