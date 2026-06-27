//! Tamanho em bytes como value object, com formatação humana.

/// Tamanho de armazenamento em bytes (base decimal para exibição).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSize(u64);

impl ByteSize {
    /// Cria a partir de uma contagem de bytes.
    #[must_use]
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Retorna a contagem de bytes.
    #[must_use]
    pub const fn as_bytes(self) -> u64 {
        self.0
    }

    /// Formata em unidade humana (ex.: "32.0 GB"). Base decimal (1000).
    #[must_use]
    pub fn humanize(self) -> String {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut value = self.0 as f64;
        let mut index = 0usize;
        while index < UNITS.len() - 1 {
            let next = value / 1000.0;
            if next < 1.0 {
                break;
            }
            // Avoid showing "1000.0 <unit>" due to rounding.
            if next >= 999.95 && index + 1 < UNITS.len() - 1 {
                value = next / 1000.0;
                index += 2;
            } else {
                value = next;
                index += 1;
            }
        }
        if index == 0 {
            format!("{} B", self.0)
        } else {
            format!("{value:.1} {}", UNITS[index])
        }
    }
}

#[cfg(test)]
mod tests;
