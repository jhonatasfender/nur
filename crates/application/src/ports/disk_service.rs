//! Porta de acesso ao disco (implementada na infraestrutura).

use crate::erros::ErroDisco;
use domain::Dispositivo;

/// Serviço de disco: enumera dispositivos e (futuramente) grava/formata.
pub trait DiskService: Send + Sync {
    /// Lista os dispositivos removíveis disponíveis.
    ///
    /// # Errors
    /// Retorna [`ErroDisco`] se o backend falhar.
    fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco>;
}
