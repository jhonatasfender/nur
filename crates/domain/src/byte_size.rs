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
        const UNIDADES: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut valor = self.0 as f64;
        let mut indice = 0;
        while valor >= 1000.0 && indice < UNIDADES.len() - 1 {
            valor /= 1000.0;
            indice += 1;
        }
        if indice == 0 {
            format!("{} B", self.0)
        } else {
            format!("{valor:.1} {}", UNIDADES[indice])
        }
    }
}

#[cfg(test)]
mod tests;
