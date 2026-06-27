//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum ErroDisco {
    /// O backend de disco está indisponível ou falhou.
    #[error("serviço de disco indisponível: {0}")]
    Indisponivel(String),
}
