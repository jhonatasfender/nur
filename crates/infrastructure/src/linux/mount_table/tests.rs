use super::MountTable;

const SAMPLE: &str = "\
/dev/sda2 / ext4 rw,relatime 0 0
/dev/sdb1 /run/media/user/USB\\040STICK vfat rw 0 0
tmpfs /tmp tmpfs rw 0 0
";

#[test]
fn finds_partition_mount_point_decoding_spaces() {
    assert_eq!(
        MountTable::mount_point_for(SAMPLE, "sdb"),
        Some("/run/media/user/USB STICK".to_owned())
    );
}

#[test]
fn matches_whole_device_when_no_partition() {
    let table = "/dev/sdc /mnt/raw ext4 rw 0 0\n";
    assert_eq!(
        MountTable::mount_point_for(table, "sdc"),
        Some("/mnt/raw".to_owned())
    );
}

#[test]
fn does_not_match_prefix_collision() {
    let table = "/dev/sdbb1 /mnt/other vfat rw 0 0\n";
    assert_eq!(MountTable::mount_point_for(table, "sdb"), None);
}

#[test]
fn returns_none_when_absent() {
    assert_eq!(MountTable::mount_point_for(SAMPLE, "sdz"), None);
}
