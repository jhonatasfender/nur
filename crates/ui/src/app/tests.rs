use super::*;
use application::errors::ScreenshotError;
use application::ports::{DeviceListState, DeviceView};
use domain::DevicePath;
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

struct CommandsFake;
impl UiCommands for CommandsFake {
    fn pick_iso(&self) {}
    fn start(&self, _device: DevicePath) {}
    fn cancel(&self) {}
}

struct NoopWriter;
impl ScreenshotWriter for NoopWriter {
    fn write(&self, _: &[u8], _: u32, _: u32, _: &Path) -> Result<(), ScreenshotError> {
        Ok(())
    }
}

#[test]
fn builder_sets_theme() {
    let app = NurApp::new(
        Arc::new(UiStateFake),
        Arc::new(CommandsFake),
        Arc::new(NoopWriter),
    )
    .with_theme(ThemePreference::Light);
    assert_eq!(app.theme(), ThemePreference::Light);
}
