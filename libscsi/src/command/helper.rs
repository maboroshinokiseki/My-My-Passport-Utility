use std::io;

use crate::{sg_io_header::HostStatus, DriverStatus, MaskedStatus, SgIoHeader};

use super::FixedSenseBuffer;

pub type BytesSenseBuffer = [u8; 255];

pub fn bytes_sense_buffer_value() -> BytesSenseBuffer {
    [0; 255]
}

pub fn fixed_sense_buffer_value() -> FixedSenseBuffer {
    FixedSenseBuffer::new()
}

pub fn check_ioctl_result(result: i32) -> io::Result<()> {
    match result {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

pub fn check_error_status<C, D>(header: &SgIoHeader<C, D, BytesSenseBuffer>) -> io::Result<()> {
    let buffer = header.sense_buffer.as_ref().map(|v| &v[..]).or(Some(&[]));

    check_error_status_any_sense(
        header.masked_status,
        header.host_status,
        header.driver_status,
        header.sense_buffer_written,
        buffer,
    )
}

pub fn check_error_status_any_sense(
    masked_status: MaskedStatus,
    host_status: u16,
    driver_status: DriverStatus,
    sense_buffer_written: u8,
    sense_buffer: Option<&[u8]>,
) -> io::Result<()> {
    let mut result = String::new();

    if !masked_status.is_empty() {
        result.push_str(&format!("masked status: {:?}. ", masked_status));
    }

    if host_status != 0 {
        result.push_str(&format!(
            "host status: {:?}. ",
            HostStatus::from(host_status)
        ));
    }

    if !driver_status.is_empty() {
        result.push_str(&format!("driver status: {:?}. ", driver_status));
    }

    if sense_buffer_written != 0 {
        result.push_str(&format!(
            "Sense data: {:02X?}",
            &sense_buffer.as_ref().unwrap()[0..sense_buffer_written as usize]
        ));
    }

    if !result.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, result.trim()));
    }

    Ok(())
}
