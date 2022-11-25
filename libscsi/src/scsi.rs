#![allow(dead_code)]

use std::{
    borrow::BorrowMut,
    fs::OpenOptions,
    io,
    mem::size_of_val,
    path::{Path, PathBuf},
};

use nix::libc;

use crate::{
    file_descriptor::{FileDescriptor, FileType},
    sg_io_header::*,
    Command, SgIoHeader,
};

#[derive(Debug)]
pub struct Scsi {
    path: PathBuf,
    file_descriptor: FileDescriptor,
    timeout: u32,
}

impl Scsi {
    pub fn new<P: AsRef<Path> + ?Sized>(path: &P, timeout: Option<u32>) -> io::Result<Scsi> {
        let path_string = String::from(path.as_ref().to_string_lossy());
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        let file_descriptor = FileDescriptor::open(&path, options)?;

        if !matches!(file_descriptor.file_type()?, FileType::BlockDevice) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{} is not a block device.", path_string),
            ));
        }

        let mut version = 0_i32;
        unsafe {
            libc::ioctl(
                file_descriptor.raw(),
                SG_GET_VERSION_NUM.try_into().unwrap(),
                &mut version,
            );
        }

        if version < 30000 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "{} is not an SCSI Generic device, or old SCSI Generic driver.",
                    path_string
                ),
            ));
        }

        Ok(Scsi {
            path: path.as_ref().to_owned(),
            file_descriptor,
            timeout: timeout.unwrap_or(SG_DEFAULT_TIMEOUT),
        })
    }

    pub fn execute_command<T: Command>(&self, command: &T) -> T::ReturnType {
        let command_buffer = command.get_command();
        let mut data_buffer = command.get_data();
        let mut sense_buffer = command.get_sense_buffer();

        let size_of_command_buffer = size_of_val(&command_buffer) as u8;
        let size_of_data_buffer = command.get_data_size();
        let size_of_sense_buffer = size_of_val(&sense_buffer) as u8;

        let pointer_of_command_buffer = Some(&command_buffer);

        let pointer_of_data_buffer = if size_of_data_buffer == 0 {
            None
        } else {
            Some(data_buffer.borrow_mut())
        };

        let pointer_of_sense_buffer = if size_of_sense_buffer == 0 {
            None
        } else {
            Some(&mut sense_buffer)
        };

        let mut sg_header = SgIoHeader {
            interface_id: b'S' as i32,
            data_direction: command.get_direction().into(),
            command_length: size_of_command_buffer,
            max_sense_buffer_length: size_of_sense_buffer,
            iovec_count: 0,
            data_length: size_of_data_buffer,
            data: pointer_of_data_buffer,
            command: pointer_of_command_buffer,
            sense_buffer: pointer_of_sense_buffer,
            timeout: self.timeout,
            flags: Flags::DEFAULT,
            pack_id: 0,
            user_pointer: 0,
            status: 0,
            masked_status: MaskedStatus::GOOD,
            message_status: 0,
            sense_buffer_written: 0,
            host_status: 0,
            driver_status: DriverStatus::OK,
            residual_count: 0,
            duration: 0,
            info: Info::OK,
        };

        let result = unsafe {
            libc::ioctl(
                self.file_descriptor.raw(),
                SG_IO.try_into().unwrap(),
                &mut sg_header,
            )
        };
        command.process_result(result, &sg_header)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn timeout(&self) -> u32 {
        self.timeout
    }
}

const SG_IO: u32 = 0x2285;
const SG_GET_VERSION_NUM: u32 = 0x2282;
const SG_DEFAULT_TIMEOUT: u32 = 60_000;
