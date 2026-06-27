use super::*;
use application::errors::ScreenshotError;
use std::path::Path;

// Gravador de teste que ignora os bytes (não toca o sistema de arquivos).
struct NoopWriter;
impl ScreenshotWriter for NoopWriter {
    fn write(&self, _: &[u8], _: u32, _: u32, _: &Path) -> Result<(), ScreenshotError> {
        Ok(())
    }
}

#[test]
fn manual_destination_increments_and_numbers() {
    let mut cap = Capturer::new(Arc::new(NoopWriter), None);
    let p1 = cap.next_destination();
    let p2 = cap.next_destination();
    assert_ne!(p1, p2);
    assert!(p1.to_string_lossy().contains("001"));
    assert!(p2.to_string_lossy().contains("002"));
}
