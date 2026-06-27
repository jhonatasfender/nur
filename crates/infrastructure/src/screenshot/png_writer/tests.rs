use super::*;

#[test]
fn writes_non_empty_png() {
    let dest = std::env::temp_dir().join("nur_png_writer_test.png");
    let _ = std::fs::remove_file(&dest);
    let rgba = vec![10u8; 4 * 4 * 4];
    PngScreenshotWriter::new()
        .write(&rgba, 4, 4, &dest)
        .unwrap();
    let meta = std::fs::metadata(&dest).unwrap();
    assert!(meta.len() > 0);
    std::fs::remove_file(&dest).unwrap();
}
