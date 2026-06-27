use super::*;

#[test]
fn themes_have_different_backgrounds() {
    assert_ne!(Palette::light().background(), Palette::dark().background());
}

#[test]
fn dark_layers_are_distinct() {
    // backdrop < card < controle formam 3 níveis distintos no tema escuro.
    let p = Palette::dark();
    assert_ne!(p.background(), p.surface());
    assert_ne!(p.surface(), p.control());
}
