use std::ffi::c_ushort;

#[derive(Debug)]
pub enum HostStatus {
    /// NO error
    Ok,
    /// Couldn't connect before timeout period
    NoConnect,
    /// BUS stayed busy through time out period
    BusBusy,
    /// TIMED OUT for other reason
    TimeOut,
    /// BAD target, device not responding?
    BadTarget,
    /// Told to abort for some other reason
    Abort,
    /// Parity error. This probably indicates a cable or termination problem
    Parity,
    /// Internal error detected in the host adapter
    Error,
    /// The SCSI bus (or this device) has been reset
    Reset,
    /// Got an interrupt we weren't expecting
    BadInterrupt,
    /// Force command past mid-layer
    Passthrough,
    /// The low level driver wants a retry
    SoftError,
    /// Retry without decrementing retry count
    ImmediateRetry,
    /// Requeue command (no immediate retry) also without decrementing the retry count
    Requeue,
    /// Something unknown
    Unknown,
}

impl From<c_ushort> for HostStatus {
    fn from(value: c_ushort) -> Self {
        match value {
            0x00 => HostStatus::Ok,
            0x01 => HostStatus::NoConnect,
            0x02 => HostStatus::BusBusy,
            0x03 => HostStatus::TimeOut,
            0x04 => HostStatus::BadTarget,
            0x05 => HostStatus::Abort,
            0x06 => HostStatus::Parity,
            0x07 => HostStatus::Error,
            0x08 => HostStatus::Reset,
            0x09 => HostStatus::BadInterrupt,
            0x0a => HostStatus::Passthrough,
            0x0b => HostStatus::SoftError,
            0x0c => HostStatus::ImmediateRetry,
            0x0d => HostStatus::Requeue,
            _ => HostStatus::Unknown,
        }
    }
}
