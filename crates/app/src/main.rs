//! Composition root do Nur: monta os adapters e abre a janela.

mod window;

use std::sync::Arc;

fn main() -> std::process::ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("erro ao criar runtime: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let _guard = runtime.enter();

    let estado = match window::UiStateAoVivo::montar() {
        Ok(estado) => Arc::new(estado),
        Err(e) => {
            eprintln!("erro ao montar estado: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    match window::abrir(estado) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
