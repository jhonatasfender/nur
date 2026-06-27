//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum DiskError {
    /// O backend de disco está indisponível ou falhou.
    #[error("serviço de disco indisponível: {0}")]
    Unavailable(String),
}
