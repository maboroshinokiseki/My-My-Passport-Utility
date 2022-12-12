use std::ffi::c_int;

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum DataDirection {
    /// e.g. a SCSI Test Unit Ready command
    None = -1,
    /// e.g. a SCSI WRITE command
    ToDevice = -2,
    /// e.g. a SCSI READ command
    FromDevice = -3,
    /// treated like FromDevice with the
    /// additional property than during indirect
    /// IO the user buffer is copied into the
    /// kernel buffers before the transfer
    ToFromDevice = -4,
    Unknown = -5,
}

impl From<DataDirection> for c_int {
    fn from(value: DataDirection) -> Self {
        value as i32 as c_int
    }
}
