use super::RawCopier;
use application::errors::WriteError;
use application::ports::{CancelFlag, ProgressSink, WriteProgress};
use std::io::Cursor;
use std::sync::Mutex;

struct RecordingSink(Mutex<Vec<u64>>);
impl ProgressSink for RecordingSink {
    fn report(&self, p: WriteProgress) {
        if let Ok(mut v) = self.0.lock() {
            v.push(p.done());
        }
    }
}

#[test]
fn copy_writes_all_bytes_and_reports_progress() {
    let data = vec![7u8; 10 * 1024 * 1024]; // 10 MiB → 3 chunks
    let mut src = Cursor::new(data.clone());
    let mut dst: Vec<u8> = Vec::new();
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    let r = RawCopier::copy(&mut src, &mut dst, data.len() as u64, &sink, &cancel);
    assert!(r.is_ok());
    assert_eq!(dst, data);
    let reported = sink.0.lock().unwrap();
    assert_eq!(*reported.last().unwrap(), data.len() as u64);
}

#[test]
fn copy_aborts_when_cancelled() {
    let data = vec![1u8; 10 * 1024 * 1024];
    let mut src = Cursor::new(data.clone());
    let mut dst: Vec<u8> = Vec::new();
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    cancel.cancel();
    let r = RawCopier::copy(&mut src, &mut dst, data.len() as u64, &sink, &cancel);
    assert!(matches!(r, Err(WriteError::Cancelled)));
}

#[test]
fn verify_detects_mismatch() {
    let original = vec![5u8; 8 * 1024 * 1024];
    let mut corrupted = original.clone();
    corrupted[100] = 0;
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    let r = RawCopier::verify(
        &mut Cursor::new(corrupted),
        &mut Cursor::new(original.clone()),
        original.len() as u64,
        &sink,
        &cancel,
    );
    assert!(matches!(r, Err(WriteError::VerificationMismatch)));
}

#[test]
fn verify_passes_on_identical() {
    let original = vec![9u8; 8 * 1024 * 1024];
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    let r = RawCopier::verify(
        &mut Cursor::new(original.clone()),
        &mut Cursor::new(original.clone()),
        original.len() as u64,
        &sink,
        &cancel,
    );
    assert!(r.is_ok());
}
