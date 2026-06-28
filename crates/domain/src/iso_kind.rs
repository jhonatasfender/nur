//! Classificação de uma imagem ISO quanto à gravação.

/// Como a ISO pode ser gravada no pendrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoKind {
    /// Isohybrid: gravável por cópia raw byte-a-byte (maioria das ISOs Linux).
    Isohybrid,
    /// Não gravável por raw neste incremento (ex.: Windows/UDF).
    Unsupported,
}

#[cfg(test)]
mod tests;
