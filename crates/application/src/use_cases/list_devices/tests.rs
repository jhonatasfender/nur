use super::*;
use domain::{ByteSize, Device, DevicePath};

struct DiskServiceFake;

#[async_trait::async_trait]
impl DiskService for DiskServiceFake {
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        Ok(vec![Device::new(
            DevicePath::new("/dev/sdb".to_owned()),
            "SanDisk Ultra".to_owned(),
            ByteSize::from_bytes(32_000_000_000),
            true,
        )])
    }
}

#[tokio::test]
async fn maps_devices_to_views() {
    let uc = ListDevices::new(Arc::new(DiskServiceFake));
    let views = uc.execute().await.unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].path(), "/dev/sdb");
    assert!(views[0].description().contains("SanDisk Ultra"));
}
