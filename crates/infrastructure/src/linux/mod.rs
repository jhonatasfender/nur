//! Adapters de IO para Linux.

mod iso_file_inspector;
mod sysfs_disk_service;

pub use iso_file_inspector::IsoFileInspector;
pub use sysfs_disk_service::SysfsDiskService;
