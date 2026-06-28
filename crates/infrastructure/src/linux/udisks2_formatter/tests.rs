use super::Udisks2Formatter;
use domain::{FilesystemKind, PartitionScheme};

#[test]
fn maps_partition_table() {
    assert_eq!(Udisks2Formatter::udisks_table(PartitionScheme::Gpt), "gpt");
    assert_eq!(Udisks2Formatter::udisks_table(PartitionScheme::Mbr), "dos");
}

#[test]
fn maps_filesystem() {
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::Fat32), "vfat");
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::Ntfs), "ntfs");
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::ExFat), "exfat");
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::Ext4), "ext4");
}
