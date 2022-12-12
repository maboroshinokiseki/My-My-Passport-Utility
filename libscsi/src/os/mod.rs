#[cfg(target_os = "linux")]
pub mod sg_io_header;

#[cfg(target_os = "windows")]
pub mod scsi_pass_through_header;
