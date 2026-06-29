//! Ferramenta de build do Nur (lints customizados).

mod bin_visibility {
    pub mod rule;
}
mod line_limit {
    pub mod rule;
}
mod pub_fields {
    pub mod rule;
}

use bin_visibility::rule::BinVisibilityRule;
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
            "bin-visibility" => Self::run_bin_visibility(),
            // Roda todas as regras (não curto-circuita: reporta tudo).
            "check" => Self::run_line_limit() & Self::run_pub_fields() & Self::run_bin_visibility(),
            _ => {
                eprintln!("uso: cargo xtask <line-limit|pub-fields|bin-visibility|check>");
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

    fn run_bin_visibility() -> bool {
        match BinVisibilityRule::check(Path::new("crates/app/src")) {
            Ok(v) if v.is_empty() => {
                println!("bin-visibility: OK");
                true
            }
            Ok(v) => {
                for item in v {
                    eprintln!("`pub` puro no binário (use pub(crate)/pub(super)): {item}");
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
