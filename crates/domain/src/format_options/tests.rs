use super::PartitionScheme;

#[test]
fn scheme_variants_distinct() {
    assert_ne!(PartitionScheme::Gpt, PartitionScheme::Mbr);
}
