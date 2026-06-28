//! Portas (traits) que a aplicação define e a infraestrutura implementa.

mod disk_service;
mod screenshot_writer;
mod ui_state;
mod write;

pub use disk_service::DiskService;
pub use screenshot_writer::ScreenshotWriter;
pub use ui_state::{DeviceListState, DeviceView, UiState};
pub use write::{
    CancelFlag, IsoSelection, ProgressSink, WritePhase, WriteProgress, WriteRequest, WriteState,
};
