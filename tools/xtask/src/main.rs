//! Ferramenta de build do Nur (lints customizados).

mod line_limit {
    pub mod rule;
}

use line_limit::rule::LineLimitRule;
use std::path::Path;

fn main() -> std::process::ExitCode {
    let command = std::env::args().nth(1).unwrap_or_default();
    match command.as_str() {
        "line-limit" => run_line_limit(),
        _ => {
            eprintln!("uso: cargo xtask line-limit");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_line_limit() -> std::process::ExitCode {
    let root = Path::new("crates");
    match LineLimitRule::check(root) {
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
