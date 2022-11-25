use bitflags::bitflags;
use nix::libc::{c_int, c_uchar, c_uint, c_ushort};

#[repr(C)]
#[derive(Debug)]
pub struct SgIoHeader<'a, C, D, S> {
    /// \[i\] 'S' for SCSI generic (required)
    pub interface_id: c_int,
    /// \[i\] data transfer direction
    pub data_direction: c_int,
    /// \[i\] SCSI command length ( <= 16 bytes)
    pub command_length: c_uchar,
    /// \[i\] max length to write to sense_buffer
    pub max_sense_buffer_length: c_uchar,
    /// \[i\] 0 implies no scatter gather
    pub iovec_count: c_ushort,
    /// \[i\] byte count of data transfer
    pub data_length: c_uint,
    /// \[i\], \[*io\] points to data transfer memory or scatter gather list
    pub data: Option<&'a mut D>,
    /// \[i\], \[*i\] points to command to perform
    pub command: Option<&'a C>,
    /// \[i\], \[*o\] points to sense_buffer memory
    pub sense_buffer: Option<&'a mut S>,
    /// \[i\] MAX_UINT->no timeout (unit: millisec)
    pub timeout: c_uint,
    /// \[i\] 0 -> default
    pub flags: Flags,
    /// \[i->o\] unused internally (normally)
    pub pack_id: c_int,
    /// \[i->o\] unused internally
    pub user_pointer: usize,
    /// \[o\] scsi status
    pub status: c_uchar,
    /// \[o\] shifted, masked scsi status
    pub masked_status: MaskedStatus,
    /// \[o\] messaging level data (optional)
    pub message_status: c_uchar,
    /// \[o\] byte count actually written to sense_buffer
    pub sense_buffer_written: c_uchar,
    /// \[o\] errors from host adapter
    pub host_status: c_ushort,
    /// \[o\] errors from software driver
    pub driver_status: DriverStatus,
    /// \[o\] data_length - actual_transferred
    pub residual_count: c_int,
    /// \[o\] time taken by cmd (unit: millisec)
    pub duration: c_uint,
    /// \[o\] auxiliary information
    pub info: Info,
}

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

bitflags! {
    pub struct Flags: c_uint {
        /// Indirect io
        const DEFAULT           = 0b0000;
        const DIRECT_IO         = 0b0001;
        const LUN_INHIBIT       = 0b0010;
        const MEMORY_MAPPED_IO  = 0b0100;
        const NO_DATA_TRANSFER  = 0b1000;
    }
}

bitflags! {
    pub struct MaskedStatus: c_uchar {
        const GOOD                  = 0x00;
        const CHECK_CONDITION       = 0x01;
        const CONDITION_GOOD        = 0x02;
        const BUSY                  = 0x04;
        const INTERMEDIATE_GOOD     = 0x08;
        const INTERMEDIATE_C_GOOD   = 0x0a;
        const RESERVATION_CONFLICT  = 0x0c;
        const COMMAND_TERMINATED    = 0x11;
        const QUEUE_FULL            = 0x14;
    }
}

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

bitflags! {
    pub struct DriverStatus: c_ushort {
        const OK            = 0x00;
        const BUSY          = 0x01;
        const SOFT          = 0x02;
        const MEDIA         = 0x03;
        const ERROR         = 0x04;
        const INVALID       = 0x05;
        const TIMEOUT       = 0x06;
        const HARD          = 0x07;
        /// Implies sense_buffer output
        const SENSE         = 0x08;
        // above status 'or'ed with one of the following suggestions
        const RETRY         = 0x10;
        const ABORT         = 0x20;
        const REMAP         = 0x30;
        const DIE           = 0x40;
        const SUGGEST_SENSE = 0x80;
    }
}

bitflags! {
    pub struct Info: c_uint {
        const OK_MASK           = 0x01;
        const OK                = 0x00;
        const CHECK             = 0x01;
        const DIRECT_IO_MASK    = 0x06;
        const INDIRECT_IO       = 0x00;
        const DIRECT_IO         = 0x02;
        const MIXED_IO          = 0x04;
    }
}

#[allow(deref_nullptr)]
#[cfg(test)]
mod tests {
    use super::SgIoHeader as SgIoHeaderGeneric;
    type SgIoHeader<'a> = SgIoHeaderGeneric<'a, (), (), ()>;

    #[test]
    fn bindgen_test_layout_sg_io_header() {
        assert_eq!(
            ::std::mem::size_of::<SgIoHeader>(),
            88usize,
            concat!("Size of: ", stringify!(SgIoHeader))
        );
        assert_eq!(
            ::std::mem::align_of::<SgIoHeader>(),
            8usize,
            concat!("Alignment of ", stringify!(SgIoHeader))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).interface_id as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(interface_id)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).data_direction as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data_direction)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).command_length as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(command_length)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SgIoHeader>())).max_sense_buffer_length as *const _ as usize
            },
            9usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(max_sense_buffer_length)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).iovec_count as *const _ as usize },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(iovec_count)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).data_length as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data_length)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).data as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).command as *const _ as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(command)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).sense_buffer as *const _ as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(sense_buffer)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).timeout as *const _ as usize },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(timeout)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).flags as *const _ as usize },
            44usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(flags)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).pack_id as *const _ as usize },
            48usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(pack_id)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).user_pointer as *const _ as usize },
            56usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(user_pointer)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).status as *const _ as usize },
            64usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(status)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).masked_status as *const _ as usize },
            65usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(masked_status)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).message_status as *const _ as usize },
            66usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(message_status)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SgIoHeader>())).sense_buffer_written as *const _ as usize
            },
            67usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(sense_buffer_written)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).host_status as *const _ as usize },
            68usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(host_status)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).driver_status as *const _ as usize },
            70usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(driver_status)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).residual_count as *const _ as usize },
            72usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(residual_count)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).duration as *const _ as usize },
            76usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(duration)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SgIoHeader>())).info as *const _ as usize },
            80usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(info)
            )
        );
    }
}
