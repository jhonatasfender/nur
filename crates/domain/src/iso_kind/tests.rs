use super::IsoKind;

#[test]
fn variants_are_distinct() {
    assert_ne!(IsoKind::Isohybrid, IsoKind::Unsupported);
    assert_eq!(IsoKind::Isohybrid, IsoKind::Isohybrid);
}
