use super::*;
use std::io::Write;

#[test]
fn aponta_arquivo_acima_do_limite() {
    let dir = std::env::temp_dir().join("nur_xtask_test_grande");
    std::fs::create_dir_all(&dir).unwrap();
    let arquivo = dir.join("grande.rs");
    let mut f = std::fs::File::create(&arquivo).unwrap();
    for _ in 0..LineLimitRule::LIMIT {
        writeln!(f, "// linha").unwrap();
    }
    let violacoes = LineLimitRule::check(&dir).unwrap();
    assert!(violacoes.iter().any(|v| v.contains("grande.rs")));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn aceita_arquivo_dentro_do_limite() {
    let dir = std::env::temp_dir().join("nur_xtask_test_pequeno");
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = std::fs::File::create(dir.join("pequeno.rs")).unwrap();
    writeln!(f, "// só uma linha").unwrap();
    let violacoes = LineLimitRule::check(&dir).unwrap();
    assert!(violacoes.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
