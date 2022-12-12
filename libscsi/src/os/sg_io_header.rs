use nix::libc::{c_int, c_uchar, c_uint, c_ushort};

use crate::{AccessFlags, AuxiliaryInfo, DriverStatus, MaskedStatus};

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
    pub flags: AccessFlags,
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
    pub info: AuxiliaryInfo,
}

#[allow(deref_nullptr)]
#[cfg(test)]
mod tests {
    use super::SgIoHeader as SgIoHeaderGeneric;
    type SgIoHeader<'a> = SgIoHeaderGeneric<'a, (), (), ()>;

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn bindgen_test_layout_sg_io_header() {
        const UNINIT: ::std::mem::MaybeUninit<SgIoHeader> = ::std::mem::MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
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
            unsafe { ::std::ptr::addr_of!((*ptr).interface_id) as usize - ptr as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(interface_id)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).data_direction) as usize - ptr as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data_direction)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).command_length) as usize - ptr as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(command_length)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).max_sense_buffer_length) as usize - ptr as usize },
            9usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(max_sense_buffer_length)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).iovec_count) as usize - ptr as usize },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(iovec_count)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).data_length) as usize - ptr as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data_length)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).data) as usize - ptr as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).command) as usize - ptr as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(command)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).sense_buffer) as usize - ptr as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(sense_buffer)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).timeout) as usize - ptr as usize },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(timeout)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).flags) as usize - ptr as usize },
            44usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(flags)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).pack_id) as usize - ptr as usize },
            48usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(pack_id)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).user_pointer) as usize - ptr as usize },
            56usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(user_pointer)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).status) as usize - ptr as usize },
            64usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).masked_status) as usize - ptr as usize },
            65usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(masked_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).message_status) as usize - ptr as usize },
            66usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(message_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).sense_buffer_written) as usize - ptr as usize },
            67usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(sense_buffer_written)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).host_status) as usize - ptr as usize },
            68usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(host_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).driver_status) as usize - ptr as usize },
            70usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(driver_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).residual_count) as usize - ptr as usize },
            72usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(residual_count)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).duration) as usize - ptr as usize },
            76usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(duration)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).info) as usize - ptr as usize },
            80usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(info)
            )
        );
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn bindgen_test_layout_sg_io_header() {
        const UNINIT: ::std::mem::MaybeUninit<SgIoHeader> = ::std::mem::MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            ::std::mem::size_of::<SgIoHeader>(),
            64usize,
            concat!("Size of: ", stringify!(SgIoHeader))
        );
        assert_eq!(
            ::std::mem::align_of::<SgIoHeader>(),
            4usize,
            concat!("Alignment of ", stringify!(SgIoHeader))
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).interface_id) as usize - ptr as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(interface_id)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).data_direction) as usize - ptr as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data_direction)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).command_length) as usize - ptr as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(command_length)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).max_sense_buffer_length) as usize - ptr as usize },
            9usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(max_sense_buffer_length)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).iovec_count) as usize - ptr as usize },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(iovec_count)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).data_length) as usize - ptr as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data_length)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).data) as usize - ptr as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(data)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).command) as usize - ptr as usize },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(command)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).sense_buffer) as usize - ptr as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(sense_buffer)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).timeout) as usize - ptr as usize },
            28usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(timeout)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).flags) as usize - ptr as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(flags)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).pack_id) as usize - ptr as usize },
            36usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(pack_id)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).user_pointer) as usize - ptr as usize },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(user_pointer)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).status) as usize - ptr as usize },
            44usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).masked_status) as usize - ptr as usize },
            45usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(masked_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).message_status) as usize - ptr as usize },
            46usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(message_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).sense_buffer_written) as usize - ptr as usize },
            47usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(sense_buffer_written)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).host_status) as usize - ptr as usize },
            48usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(host_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).driver_status) as usize - ptr as usize },
            50usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(driver_status)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).residual_count) as usize - ptr as usize },
            52usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(residual_count)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).duration) as usize - ptr as usize },
            56usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(duration)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).info) as usize - ptr as usize },
            60usize,
            concat!(
                "Offset of field: ",
                stringify!(SgIoHeader),
                "::",
                stringify!(info)
            )
        );
    }
}
