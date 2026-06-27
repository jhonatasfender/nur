//! Portas (traits) que a aplicação define e a infraestrutura implementa.

mod disk_service;
mod ui_state;

pub use disk_service::DiskService;
pub use ui_state::{DispositivoView, UiState};
