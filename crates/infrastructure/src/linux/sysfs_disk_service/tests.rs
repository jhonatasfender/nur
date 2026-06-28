use super::*;

#[test]
fn maps_usb_block_device() {
    let device = SysfsDiskService::build_device(
        "sdb",
        "/sys/devices/pci0000:00/usb1/1-1/host4/target4:0:0/4:0:0:0/block/sdb",
        62_500_000,
        true,
        "SanDisk Ultra\n",
    )
    .expect("dispositivo USB deve mapear");
    assert_eq!(device.path().as_str(), "/dev/sdb");
    assert_eq!(device.model(), "SanDisk Ultra");
    assert!(device.removable());
    assert_eq!(device.size().as_bytes(), 62_500_000 * 512);
}

#[test]
fn skips_non_usb_disk() {
    let mapped = SysfsDiskService::build_device(
        "nvme0n1",
        "/sys/devices/pci0000:00/0000:00:1d.0/nvme/nvme0/block/nvme0n1",
        1_000_000_000,
        false,
        "Samsung SSD",
    );
    assert!(mapped.is_none());
}
