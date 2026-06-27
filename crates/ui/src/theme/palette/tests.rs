use super::*;

#[test]
fn themes_have_different_backgrounds() {
    assert_ne!(Palette::light().background(), Palette::dark().background());
}

#[test]
fn success_is_green_in_both_themes() {
    // Verde de sucesso é o mesmo token (#16A34A) nos dois temas.
    assert_eq!(Palette::light().success(), Palette::dark().success());
}
