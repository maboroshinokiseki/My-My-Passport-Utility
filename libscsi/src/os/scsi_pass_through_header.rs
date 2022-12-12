use windows::Win32::Storage::IscsiDisc::SCSI_PASS_THROUGH_DIRECT;

#[repr(C)]
pub struct ScsiPassThroughDirectWrapper<S> {
    pub scsi_pass_through: SCSI_PASS_THROUGH_DIRECT,
    pub sense: S,
}

impl<S> Default for ScsiPassThroughDirectWrapper<S> {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
