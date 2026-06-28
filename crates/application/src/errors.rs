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

/// Falhas ao inspecionar/ler a imagem ISO.
#[derive(Debug, thiserror::Error)]
pub enum IsoError {
    /// Falha de IO ao ler a ISO.
    #[error("falha ao ler a ISO: {0}")]
    Io(String),
}

/// Falhas ao gravar a imagem no dispositivo.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// O polkit negou a autorização para abrir o dispositivo.
    #[error("autorização negada")]
    Unauthorized,
    /// O dispositivo está em uso (montado ou aberto por outro processo).
    #[error("dispositivo ocupado")]
    DeviceBusy,
    /// O dispositivo é menor que a imagem.
    #[error("o dispositivo é menor que a imagem")]
    DeviceTooSmall,
    /// Falha de IO durante a gravação ou a leitura de verificação.
    #[error("falha de gravação: {0}")]
    Io(String),
    /// A releitura do dispositivo não bate com a imagem.
    #[error("verificação falhou: o conteúdo gravado difere da imagem")]
    VerificationMismatch,
    /// O usuário cancelou a operação.
    #[error("operação cancelada")]
    Cancelled,
}

#[cfg(test)]
mod tests;
