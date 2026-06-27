use super::*;
use application::errors::ScreenshotError;
use application::ports::DeviceView;
use std::path::Path;

struct UiStateFake;
impl UiState for UiStateFake {
    fn devices(&self) -> Vec<DeviceView> {
        vec![DeviceView::new(
            "/dev/sdb".to_owned(),
            "Teste — 32.0 GB (/dev/sdb)".to_owned(),
        )]
    }
}

struct NoopWriter;
impl ScreenshotWriter for NoopWriter {
    fn write(&self, _: &[u8], _: u32, _: u32, _: &Path) -> Result<(), ScreenshotError> {
        Ok(())
    }
}

#[test]
fn builder_sets_theme() {
    let app =
        NurApp::new(Arc::new(UiStateFake), Arc::new(NoopWriter)).with_theme(ThemePreference::Light);
    assert_eq!(app.theme(), ThemePreference::Light);
}
