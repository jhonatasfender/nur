use super::*;

#[test]
fn toggles_between_light_and_dark() {
    assert_eq!(ThemePreference::Light.toggle(), ThemePreference::Dark);
    assert_eq!(ThemePreference::Dark.toggle(), ThemePreference::Light);
}

#[test]
fn preference_resolves_palette() {
    assert_eq!(ThemePreference::Dark.palette(), Palette::dark());
}
