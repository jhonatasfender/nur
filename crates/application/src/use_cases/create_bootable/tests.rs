use super::CreateBootable;
use crate::errors::{IsoError, WriteError};
use crate::ports::{
    BootableWriter, CancelFlag, IsoInspector, ProgressSink, WriteProgress, WriteRequest,
};
use domain::{DevicePath, IsoKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct FixedInspector(IsoKind);
#[async_trait::async_trait]
impl IsoInspector for FixedInspector {
    async fn classify(&self, _iso: &Path) -> Result<IsoKind, IsoError> {
        Ok(self.0)
    }
}

struct SpyWriter(Arc<AtomicBool>);
#[async_trait::async_trait]
impl BootableWriter for SpyWriter {
    async fn write(
        &self,
        _r: &WriteRequest,
        _s: Arc<dyn ProgressSink>,
        _c: &CancelFlag,
    ) -> Result<(), WriteError> {
        self.0.store(true, Ordering::SeqCst);
        Ok(())
    }
}

struct NoopSink;
impl ProgressSink for NoopSink {
    fn report(&self, _p: WriteProgress) {}
}

fn request() -> WriteRequest {
    WriteRequest::new(
        PathBuf::from("/tmp/x.iso"),
        DevicePath::new("/dev/sdb".to_owned()),
    )
}

#[tokio::test]
async fn isohybrid_invokes_writer() {
    let called = Arc::new(AtomicBool::new(false));
    let uc = CreateBootable::new(
        Arc::new(FixedInspector(IsoKind::Isohybrid)),
        Arc::new(SpyWriter(Arc::clone(&called))),
    );
    let result = uc
        .execute(request(), Arc::new(NoopSink), CancelFlag::new())
        .await;
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn unsupported_is_rejected_without_writing() {
    let called = Arc::new(AtomicBool::new(false));
    let uc = CreateBootable::new(
        Arc::new(FixedInspector(IsoKind::Unsupported)),
        Arc::new(SpyWriter(Arc::clone(&called))),
    );
    let result = uc
        .execute(request(), Arc::new(NoopSink), CancelFlag::new())
        .await;
    assert!(result.is_err());
    assert!(!called.load(Ordering::SeqCst));
}
