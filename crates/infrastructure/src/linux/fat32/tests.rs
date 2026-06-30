use super::Fat32;
use domain::VolumeLabel;
use std::io::Cursor;

#[test]
fn formats_fat32_with_label() {
    // FAT32 exige ~33 MiB+; usamos 64 MiB.
    let mut buf = Cursor::new(vec![0u8; 64 * 1024 * 1024]);
    let label = VolumeLabel::parse("BOOTUSB").unwrap();
    Fat32::format(&mut buf, &label).unwrap();

    let fs = fatfs::FileSystem::new(&mut buf, fatfs::FsOptions::new()).unwrap();
    assert_eq!(fs.fat_type(), fatfs::FatType::Fat32);
    assert_eq!(fs.volume_label().trim_end(), "BOOTUSB");
}
