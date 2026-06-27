//! Regra: proíbe campos `pub` em structs (encapsulamento — usar getters).

use std::path::Path;

/// Garante que nenhum struct exponha campos `pub` (devem ser privados + getters).
pub struct PubFieldsRule;

impl PubFieldsRule {
    // Palavras que podem seguir `pub ` mas são itens, não campos.
    const KEYWORDS: [&str; 11] = [
        "fn", "const", "use", "mod", "struct", "enum", "trait", "type", "static", "async", "unsafe",
    ];

    /// Varre `root` recursivamente e retorna `arquivo:linha` de cada campo `pub`.
    pub fn check(root: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut hits = Vec::new();
        Self::scan(root, &mut hits)?;
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
                    if Self::is_pub_field(line) {
                        acc.push(format!("{}:{}", path.display(), index + 1));
                    }
                }
            }
        }
        Ok(())
    }

    // Detecta uma linha do tipo `pub <ident>:` (campo público), ignorando
    // `pub fn/const/struct/...`, `pub use` e `pub(crate)`.
    fn is_pub_field(line: &str) -> bool {
        let Some(rest) = line.trim_start().strip_prefix("pub ") else {
            return false;
        };
        let word: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if word.is_empty() || Self::KEYWORDS.contains(&word.as_str()) {
            return false;
        }
        rest[word.len()..].trim_start().starts_with(':')
    }
}

#[cfg(test)]
mod tests;
