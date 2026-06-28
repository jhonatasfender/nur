use super::*;
use application::errors::ScreenshotError;
use application::ports::{DeviceListState, DeviceView};
use std::path::Path;

struct UiStateFake;
impl UiState for UiStateFake {
    fn device_list(&self) -> DeviceListState {
        DeviceListState::Ready(vec![DeviceView::new(
            "/dev/sdb".to_owned(),
            "Teste — 32.0 GB (/dev/sdb)".to_owned(),
        )])
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
