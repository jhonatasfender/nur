//! Cenários de demonstração para captura/validação visual.
//!
//! A escolha do cenário (origem: env `NUR_DEMO`) é feita no composition root e
//! injetada via [`NurApp::with_demo`]; aqui só aplicamos o estado na tela.

use super::{Mode, NurApp, Phase};

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
        match scenario {
            DemoScenario::Ready => {
                self.selected = Some(0);
                self.iso_selected = true;
            }
            DemoScenario::Modal => {
                self.selected = Some(0);
                self.iso_selected = true;
                self.modal_open = true;
                "APAGAR".clone_into(&mut self.confirm_text);
            }
            DemoScenario::Running => {
                self.selected = Some(0);
                self.iso_selected = true;
                self.phase = Phase::Working;
                self.progress = 0.42;
            }
            DemoScenario::Format => {
                self.selected = Some(0);
                self.mode = Mode::Format;
            }
        }
    }
}
