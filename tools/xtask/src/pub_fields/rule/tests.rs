use super::*;

#[test]
fn flags_public_field() {
    assert!(PubFieldsRule::is_pub_field("    pub name: String,"));
    assert!(PubFieldsRule::is_pub_field("pub size: [usize; 2],"));
}

#[test]
fn ignores_public_items() {
    assert!(!PubFieldsRule::is_pub_field("    pub fn foo() {"));
    assert!(!PubFieldsRule::is_pub_field("pub struct Foo {"));
    assert!(!PubFieldsRule::is_pub_field(
        "    pub const LIMIT: usize = 200;"
    ));
    assert!(!PubFieldsRule::is_pub_field("pub use foo::Bar;"));
    assert!(!PubFieldsRule::is_pub_field("pub mod theme;"));
}

#[test]
fn ignores_restricted_visibility() {
    assert!(!PubFieldsRule::is_pub_field("    pub(crate) name: String,"));
    assert!(!PubFieldsRule::is_pub_field("    name: String,"));
}
