use super::*;
use std::io::Write;

#[test]
fn points_out_file_above_limit() {
    let dir = std::env::temp_dir().join("nur_xtask_test_large");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("large.rs");
    let mut f = std::fs::File::create(&file).unwrap();
    for _ in 0..LineLimitRule::LIMIT {
        writeln!(f, "// linha").unwrap();
    }
    let violations = LineLimitRule::check(&dir).unwrap();
    assert!(violations.iter().any(|v| v.contains("large.rs")));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn accepts_file_within_limit() {
    let dir = std::env::temp_dir().join("nur_xtask_test_small");
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = std::fs::File::create(dir.join("small.rs")).unwrap();
    writeln!(f, "// só uma linha").unwrap();
    let violations = LineLimitRule::check(&dir).unwrap();
    assert!(violations.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
