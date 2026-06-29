//! Regra: o crate binário não pode ter `pub` "puro" (não tem API; use `pub(crate)`).
//!
//! Um binário não é consumido por ninguém, então todo `pub` largo ali é
//! visibilidade maior que o necessário. Exige `pub(crate)`/`pub(super)`/privado.

use std::path::Path;

/// Garante que o crate `app` (binário) não exponha itens com `pub` puro.
pub struct BinVisibilityRule;

impl BinVisibilityRule {
    /// Varre `root` (a `src` do binário) e retorna `arquivo:linha` de cada `pub` puro.
    pub fn check(root: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut hits = Vec::new();
        if root.exists() {
            Self::scan(root, &mut hits)?;
        }
        Ok(hits)
    }

    fn scan(dir: &Path, acc: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                Self::scan(&path, acc)?;
            } else if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(&path)?;
                for (index, line) in content.lines().enumerate() {
                    if Self::is_bare_pub(line) {
                        acc.push(format!("{}:{}", path.display(), index + 1));
                    }
                }
            }
        }
        Ok(())
    }

    // `pub ` puro (seguido de espaço) — `pub(crate)`/`pub(super)` começam com `pub(`
    // e não casam; linhas de comentário (`//`, `///`) também não.
    fn is_bare_pub(line: &str) -> bool {
        line.trim_start().starts_with("pub ")
    }
}

#[cfg(test)]
mod tests;
