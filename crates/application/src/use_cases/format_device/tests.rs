use super::FormatDevice;
use crate::errors::FormatError;
use crate::ports::{DeviceFormatter, FormatOptions, ProgressSink, WriteProgress};
use domain::{DevicePath, PartitionScheme, VolumeLabel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct SpyFormatter(Arc<AtomicBool>);
#[async_trait::async_trait]
impl DeviceFormatter for SpyFormatter {
    async fn format(&self, _d: &DevicePath, _o: &FormatOptions) -> Result<(), FormatError> {
        self.0.store(true, Ordering::SeqCst);
        Ok(())
    }
}

struct NoopSink;
impl ProgressSink for NoopSink {
    fn report(&self, _p: WriteProgress) {}
}

fn options() -> FormatOptions {
    FormatOptions::new(
        PartitionScheme::Gpt,
        VolumeLabel::parse("BOOTUSB").unwrap(),
        true,
    )
}

#[tokio::test]
async fn execute_delegates_to_formatter() {
    let called = Arc::new(AtomicBool::new(false));
    let uc = FormatDevice::new(Arc::new(SpyFormatter(Arc::clone(&called))));
    let result = uc
        .execute(
            DevicePath::new("/dev/sdb".to_owned()),
            options(),
            Arc::new(NoopSink),
        )
        .await;
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}
