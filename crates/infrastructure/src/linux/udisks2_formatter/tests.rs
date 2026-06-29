use super::Udisks2Formatter;
use application::errors::FormatError;
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

#[test]
fn classify_err_maps_polkit_and_busy() {
    assert!(matches!(
        Udisks2Formatter::classify_err("GDBus.Error: NotAuthorized", "exFAT"),
        FormatError::Unauthorized
    ));
    assert!(matches!(
        Udisks2Formatter::classify_err("device is busy", "exFAT"),
        FormatError::DeviceBusy
    ));
}

#[test]
fn classify_err_tool_missing_only_with_filesystem() {
    // No passo de mkfs (fs_human preenchido), "not found" indica ferramenta ausente.
    assert!(matches!(
        Udisks2Formatter::classify_err("mkfs.exfat: command not found", "exFAT"),
        FormatError::ToolMissing(fs) if fs == "exFAT"
    ));
    // Nas chamadas de tabela/partição (fs_human vazio), NÃO é ToolMissing.
    assert!(matches!(
        Udisks2Formatter::classify_err("something not found", ""),
        FormatError::Backend(_)
    ));
}
