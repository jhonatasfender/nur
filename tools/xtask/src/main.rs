//! Ferramenta de build do Nur (lints customizados).

mod line_limit {
    pub mod rule;
}

use line_limit::rule::LineLimitRule;

fn main() -> std::process::ExitCode {
    let comando = std::env::args().nth(1).unwrap_or_default();
    match comando.as_str() {
        "line-limit" => executar_line_limit(),
        _ => {
            eprintln!("uso: cargo xtask line-limit");
            std::process::ExitCode::FAILURE
        }
    }
}

fn executar_line_limit() -> std::process::ExitCode {
    let raiz = Path::new("crates");
    match LineLimitRule::check(raiz) {
        Ok(v) if v.is_empty() => {
            println!("line-limit: OK");
            std::process::ExitCode::SUCCESS
        }
        Ok(v) => {
            for item in v {
                eprintln!("EXCEDE 199 linhas: {item}");
            }
            std::process::ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

use std::path::Path;
