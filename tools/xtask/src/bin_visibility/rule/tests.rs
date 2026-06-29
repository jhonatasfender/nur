use super::BinVisibilityRule;

#[test]
fn flags_bare_pub() {
    assert!(BinVisibilityRule::is_bare_pub("pub fn x() {}"));
    assert!(BinVisibilityRule::is_bare_pub("    pub struct S;"));
    assert!(BinVisibilityRule::is_bare_pub("pub use crate::X;"));
}

#[test]
fn accepts_restricted_or_private() {
    assert!(!BinVisibilityRule::is_bare_pub("pub(crate) fn x() {}"));
    assert!(!BinVisibilityRule::is_bare_pub("    pub(super) struct S;"));
    assert!(!BinVisibilityRule::is_bare_pub("fn private() {}"));
    assert!(!BinVisibilityRule::is_bare_pub("// pub fn doc example"));
    assert!(!BinVisibilityRule::is_bare_pub("/// pub fn em doc"));
}
