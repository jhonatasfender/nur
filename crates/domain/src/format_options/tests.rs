use super::{FilesystemKind, PartitionScheme};

#[test]
fn scheme_variants_distinct() {
    assert_ne!(PartitionScheme::Gpt, PartitionScheme::Mbr);
}

#[test]
fn filesystem_variants_distinct() {
    assert_ne!(FilesystemKind::Fat32, FilesystemKind::Ntfs);
    assert_ne!(FilesystemKind::ExFat, FilesystemKind::Ext4);
}
