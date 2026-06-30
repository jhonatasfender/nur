//! Value object da formatação: esquema de partição.

/// Esquema da tabela de partição.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionScheme {
    /// GUID Partition Table.
    Gpt,
    /// Master Boot Record (DOS).
    Mbr,
}

#[cfg(test)]
mod tests;
