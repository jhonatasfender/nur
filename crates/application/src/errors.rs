//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum DiskError {
    /// O backend de disco está indisponível ou falhou.
    #[error("serviço de disco indisponível: {0}")]
    Unavailable(String),
}

/// Falhas ao gravar uma captura de tela.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    /// A gravação da imagem no destino falhou.
    #[error("falha ao gravar captura: {0}")]
    Write(String),
}
