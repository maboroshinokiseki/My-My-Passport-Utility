#![allow(dead_code)]

use std::{
    fs::{File, OpenOptions},
    path::Path,
};

#[cfg(target_os = "windows")]
use std::os::windows::prelude::RawHandle;

#[derive(Debug)]
pub struct FileDescriptor {
    file: File,
    path: String,
}

impl FileDescriptor {
    pub fn open<P: AsRef<Path> + ?Sized>(
        path: &P,
        options: OpenOptions,
    ) -> crate::Result<FileDescriptor> {
        let path_string = String::from(path.as_ref().to_string_lossy());
        let file = options.open(path)?;

        Ok(FileDescriptor {
            file,
            path: path_string,
        })
    }

    #[cfg(target_os = "linux")]
    pub fn is_block(&self) -> crate::Result<bool> {
        use std::os::unix::prelude::FileTypeExt;

        let file_type = self.file.metadata()?.file_type();
        Ok(file_type.is_block_device())
    }

    #[cfg(target_os = "windows")]
    pub fn is_block(&self) -> crate::Result<bool> {
        use std::io;
        use std::mem::size_of;

        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::FileSystem::{
            FILE_DEVICE_CD_ROM, FILE_DEVICE_DISK, FILE_DEVICE_DVD, FILE_DEVICE_TAPE,
            FILE_DEVICE_TYPE,
        };
        use windows::Win32::System::Ioctl::STORAGE_DEVICE_NUMBER;
        use windows::Win32::System::{Ioctl::IOCTL_STORAGE_GET_DEVICE_NUMBER, IO::DeviceIoControl};

        let handle = self.raw();

        let mut device_number = STORAGE_DEVICE_NUMBER::default();

        let mut bytes_returned = 0;

        let success = unsafe {
            DeviceIoControl(
                HANDLE(handle as isize),
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some(&mut device_number as *mut _ as _),
                size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success == false {
            Err(io::Error::last_os_error())?;
        }

        match FILE_DEVICE_TYPE(device_number.DeviceType) {
            FILE_DEVICE_CD_ROM | FILE_DEVICE_DISK | FILE_DEVICE_TAPE | FILE_DEVICE_DVD => Ok(true),
            _ => Ok(false),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn raw(&self) -> i32 {
        use std::os::unix::prelude::AsRawFd;

        self.file.as_raw_fd()
    }

    #[cfg(target_os = "windows")]
    pub fn raw(&self) -> RawHandle {
        use std::os::windows::prelude::AsRawHandle;

        self.file.as_raw_handle()
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}
