//! Composition root do Nur: monta os adapters e abre a janela.

mod window;

fn main() -> std::process::ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("erro ao criar runtime: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    // O runtime fica vivo durante a janela; a task de polling roda nas worker
    // threads enquanto o eframe bloqueia a main thread.
    match window::Window::open(runtime.handle().clone()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
