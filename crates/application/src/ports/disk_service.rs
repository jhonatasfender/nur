//! Porta de acesso ao disco (implementada na infraestrutura).

use crate::errors::DiskError;
use domain::Device;

/// Serviço de disco: enumera dispositivos e (futuramente) grava/formata.
pub trait DiskService: Send + Sync {
    /// Lista os dispositivos removíveis disponíveis.
    ///
    /// # Errors
    /// Retorna [`DiskError`] se o backend falhar.
    fn list_devices(&self) -> Result<Vec<Device>, DiskError>;
}
