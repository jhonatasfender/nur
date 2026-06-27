use super::*;
use application::ports::DispositivoView;

struct UiStateFake;
impl UiState for UiStateFake {
    fn dispositivos(&self) -> Vec<DispositivoView> {
        vec![DispositivoView {
            caminho: "/dev/sdb".to_owned(),
            descricao: "Teste — 32.0 GB (/dev/sdb)".to_owned(),
        }]
    }
}

#[test]
fn builder_define_tema() {
    let app = NurApp::new(Arc::new(UiStateFake)).com_tema(ThemePreference::Claro);
    assert_eq!(app.tema(), ThemePreference::Claro);
}
