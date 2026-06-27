//! Regra de limite de linhas por arquivo `.rs`.

use std::path::Path;

/// Verifica que nenhum arquivo `.rs` ultrapassa o limite de linhas.
pub struct LineLimitRule;

impl LineLimitRule {
    /// Limite (exclusivo): arquivos com `LIMIT` ou mais linhas violam a regra.
    pub const LIMIT: usize = 200;

    /// Varre `root` recursivamente e retorna os caminhos que excedem o limite.
    pub fn check(root: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut violations = Vec::new();
        Self::scan(root, &mut violations)?;
        Ok(violations)
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
                let lines = std::fs::read_to_string(&path)?.lines().count();
                if lines >= Self::LIMIT {
                    acc.push(format!("{} ({lines} linhas)", path.display()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
