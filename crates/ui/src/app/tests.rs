use super::*;
use application::ports::DeviceView;

struct UiStateFake;
impl UiState for UiStateFake {
    fn devices(&self) -> Vec<DeviceView> {
        vec![DeviceView::new(
            "/dev/sdb".to_owned(),
            "Teste — 32.0 GB (/dev/sdb)".to_owned(),
        )]
    }
}

#[test]
fn builder_sets_theme() {
    let app = NurApp::new(Arc::new(UiStateFake)).with_theme(ThemePreference::Light);
    assert_eq!(app.theme(), ThemePreference::Light);
}
