//! Ferramenta de build do Nur (lints customizados).

mod line_limit {
    pub mod rule;
}
mod pub_fields {
    pub mod rule;
}

use line_limit::rule::LineLimitRule;
use pub_fields::rule::PubFieldsRule;
use std::path::Path;

fn main() -> std::process::ExitCode {
    let command = std::env::args().nth(1).unwrap_or_default();
    if Xtask::run(&command) {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// Despachante das regras de qualidade customizadas.
struct Xtask;

impl Xtask {
    /// Executa o comando solicitado, devolvendo `true` se tudo passou.
    fn run(command: &str) -> bool {
        match command {
            "line-limit" => Self::run_line_limit(),
            "pub-fields" => Self::run_pub_fields(),
            // Roda todas as regras (não curto-circuita: reporta tudo).
            "check" => Self::run_line_limit() & Self::run_pub_fields(),
            _ => {
                eprintln!("uso: cargo xtask <line-limit|pub-fields|check>");
                false
            }
        }
    }

    fn run_line_limit() -> bool {
        match LineLimitRule::check(Path::new("crates")) {
            Ok(v) if v.is_empty() => {
                println!("line-limit: OK");
                true
            }
            Ok(v) => {
                for item in v {
                    eprintln!("EXCEDE 199 linhas: {item}");
                }
                false
            }
            Err(e) => {
                eprintln!("erro: {e}");
                false
            }
        }
    }

    fn run_pub_fields() -> bool {
        match PubFieldsRule::check(Path::new("crates")) {
            Ok(v) if v.is_empty() => {
                println!("pub-fields: OK");
                true
            }
            Ok(v) => {
                for item in v {
                    eprintln!("campo pub proibido (use campo privado + getter): {item}");
                }
                false
            }
            Err(e) => {
                eprintln!("erro: {e}");
                false
            }
        }
    }
}
