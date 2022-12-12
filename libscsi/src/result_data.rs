use std::io;

use crate::{command::sense::Sense, error, DriverStatus, HostStatus, MaskedStatus};

pub struct ResultData<'a, D, S> {
    pub ioctl_result: i32,
    pub transfered_data_length: usize,
    pub data: Option<&'a mut D>,
    pub transfered_sense_length: usize,
    pub sense_buffer: Option<&'a mut S>,
    /// Linux only
    pub masked_status: MaskedStatus,
    /// Linux only
    pub host_status: HostStatus,
    /// Linux only
    pub driver_status: DriverStatus,
}

impl<D, S> ResultData<'_, D, S>
where
    S: Sense,
{
    pub fn check_common_error(&self) -> crate::Result<()> {
        let mut result = String::new();

        if !self.masked_status.is_empty() {
            result.push_str(&format!("masked status: {:?}. ", self.masked_status));
        }

        if !matches!(self.host_status, HostStatus::Ok) {
            result.push_str(&format!("host status: {:?}. ", self.host_status));
        }

        if !self.driver_status.is_empty() {
            result.push_str(&format!("driver status: {:?}. ", self.driver_status));
        }

        if self.transfered_sense_length != 0 {
            result.push_str(&format!(
                "Sense data: {:02X?}",
                self.sense_buffer.as_ref().unwrap().as_byte_slice()
            ));
        }

        if !result.is_empty() {
            return Err(crate::Error::Other(result));
        }

        Ok(())
    }
}

impl<D, S> ResultData<'_, D, S> {
    pub fn check_ioctl_error(&self) -> error::Result<()> {
        match self.ioctl_result {
            0 => Ok(()),
            _ => Err(error::Error::IO(io::Error::last_os_error())),
        }
    }
}
