use super::*;

#[test]
fn alterna_entre_claro_e_escuro() {
    assert_eq!(ThemePreference::Claro.alternar(), ThemePreference::Escuro);
    assert_eq!(ThemePreference::Escuro.alternar(), ThemePreference::Claro);
}

#[test]
fn preferencia_resolve_palette() {
    assert_eq!(ThemePreference::Escuro.palette(), Palette::escura());
}
