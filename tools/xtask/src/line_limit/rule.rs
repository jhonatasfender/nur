//! Regra de limite de linhas por arquivo `.rs`.

use std::path::Path;

/// Verifica que nenhum arquivo `.rs` ultrapassa o limite de linhas.
pub struct LineLimitRule;

impl LineLimitRule {
    /// Limite (exclusivo): arquivos com `LIMIT` ou mais linhas violam a regra.
    pub const LIMIT: usize = 200;

    /// Varre `raiz` recursivamente e retorna os caminhos que excedem o limite.
    pub fn check(raiz: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut violacoes = Vec::new();
        Self::varrer(raiz, &mut violacoes)?;
        Ok(violacoes)
    }

    fn varrer(dir: &Path, acc: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entrada in std::fs::read_dir(dir)? {
            let caminho = entrada?.path();
            if caminho.is_dir() {
                if caminho.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                Self::varrer(&caminho, acc)?;
            } else if caminho.extension().is_some_and(|e| e == "rs") {
                let linhas = std::fs::read_to_string(&caminho)?.lines().count();
                if linhas >= Self::LIMIT {
                    acc.push(format!("{} ({linhas} linhas)", caminho.display()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
