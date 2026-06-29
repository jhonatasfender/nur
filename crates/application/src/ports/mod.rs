//! Portas (traits) que a aplicação define e a infraestrutura implementa.

mod bootable_writer;
mod device_browser;
mod device_formatter;
mod disk_service;
mod format;
mod iso_inspector;
mod screenshot_writer;
mod ui_commands;
mod ui_state;
mod write;

pub use bootable_writer::BootableWriter;
pub use device_browser::DeviceBrowser;
pub use device_formatter::DeviceFormatter;
pub use disk_service::DiskService;
pub use format::FormatOptions;
pub use iso_inspector::IsoInspector;
pub use screenshot_writer::ScreenshotWriter;
pub use ui_commands::UiCommands;
pub use ui_state::{DeviceListState, DeviceView, UiState};
pub use write::{
    CancelFlag, IsoSelection, ProgressSink, WritePhase, WriteProgress, WriteRequest, WriteState,
};
