//! Cenários de demonstração para captura/validação visual.
//!
//! A escolha do cenário (origem: env `NUR_DEMO`) é feita no composition root e
//! injetada via [`NurApp::with_demo`]; aqui trocamos o estado lido pela UI por
//! um [`DemoUiState`] com valores canônicos.

use super::{Mode, NurApp};
use application::ports::{DeviceListState, DeviceView, IsoSelection, UiState, WriteState};
use domain::{ByteSize, IsoKind};
use std::sync::Arc;

/// Estado inicial pré-montado para validar a UI de forma headless.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoScenario {
    /// Dispositivo e ISO selecionados, pronto para gravar.
    Ready,
    /// Modal de confirmação aberto.
    Modal,
    /// Operação em andamento (gravando).
    Running,
    /// Modo apenas formatar.
    Format,
}

// Estado canônico para os cenários de demonstração (sem IO real).
struct DemoUiState {
    scenario: DemoScenario,
}

impl UiState for DemoUiState {
    fn device_list(&self) -> DeviceListState {
        DeviceListState::Ready(vec![DeviceView::new(
            "/dev/sdb".to_owned(),
            "SanDisk Ultra \u{2014} 32.0 GB (/dev/sdb)".to_owned(),
        )])
    }

    fn selected_iso(&self) -> Option<IsoSelection> {
        match self.scenario {
            DemoScenario::Format => None,
            _ => Some(IsoSelection::new(
                "ubuntu-24.04-desktop-amd64.iso".to_owned(),
                ByteSize::from_bytes(6_200_000_000),
                IsoKind::Isohybrid,
            )),
        }
    }

    fn write_state(&self) -> WriteState {
        match self.scenario {
            DemoScenario::Running => WriteState::Writing {
                done: 42,
                total: 100,
            },
            _ => WriteState::Idle,
        }
    }
}

impl NurApp {
    /// Builder: aplica um cenário de demonstração, se houver.
    #[must_use]
    pub fn with_demo(mut self, scenario: Option<DemoScenario>) -> Self {
        if let Some(scenario) = scenario {
            self.apply_demo(scenario);
        }
        self
    }

    fn apply_demo(&mut self, scenario: DemoScenario) {
        self.state = Arc::new(DemoUiState { scenario });
        self.selected = Some(0);
        match scenario {
            DemoScenario::Modal => {
                self.modal_open = true;
                "APAGAR".clone_into(&mut self.confirm_text);
            }
            DemoScenario::Format => {
                self.mode = Mode::Format;
            }
            DemoScenario::Ready | DemoScenario::Running => {}
        }
    }
}
