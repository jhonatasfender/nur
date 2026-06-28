//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum DiskError {
    /// O backend de disco está indisponível ou não respondeu.
    #[error("serviço de disco indisponível: {0}")]
    Unavailable(String),
    /// Falha do backend ao consultar/operar dispositivos.
    #[error("falha no backend de disco: {0}")]
    Backend(String),
}

/// Falhas ao gravar uma captura de tela.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    /// A gravação da imagem no destino falhou.
    #[error("falha ao gravar captura: {0}")]
    Write(String),
}
