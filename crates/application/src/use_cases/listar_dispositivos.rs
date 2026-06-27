//! Caso de uso: listar dispositivos disponíveis para exibição.

use crate::erros::ErroDisco;
use crate::ports::{DiskService, DispositivoView};
use std::sync::Arc;

/// Lista dispositivos e os projeta para a UI.
pub struct ListarDispositivos {
    servico: Arc<dyn DiskService>,
}

impl ListarDispositivos {
    /// Cria o caso de uso com a porta de disco injetada.
    #[must_use]
    pub fn new(servico: Arc<dyn DiskService>) -> Self {
        Self { servico }
    }

    /// Executa a listagem e mapeia para [`DispositivoView`].
    ///
    /// # Errors
    /// Propaga [`ErroDisco`] do backend.
    pub fn executar(&self) -> Result<Vec<DispositivoView>, ErroDisco> {
        let dispositivos = self.servico.listar_dispositivos()?;
        Ok(dispositivos
            .into_iter()
            .map(|d| DispositivoView {
                caminho: d.caminho().as_str().to_owned(),
                descricao: d.descricao(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests;
