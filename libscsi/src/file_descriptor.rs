#![allow(dead_code)]

use std::{
    fs::{File, OpenOptions},
    io,
    os::unix::prelude::AsRawFd,
    path::Path,
};

#[derive(Debug)]
pub enum FileType {
    Directory,
    CharacterDevice,
    BlockDevice,
    RegularFile,
    SymbolicLink,
    Other,
}

#[derive(Debug)]
pub struct FileDescriptor {
    file: File,
    path: String,
}

impl FileDescriptor {
    pub fn open<P: AsRef<Path> + ?Sized>(
        path: &P,
        options: OpenOptions,
    ) -> io::Result<FileDescriptor> {
        let path_string = String::from(path.as_ref().to_string_lossy());
        let file = options.open(path)?;

        Ok(FileDescriptor {
            file,
            path: path_string,
        })
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        const FILE_TYPE_BITS: u32 = 0xF000;
        const CHARACTER: u32 = 0x2000;
        const DIRECTORY: u32 = 0x4000;
        const BLOCK: u32 = 0x6000;
        const REGULAR: u32 = 0x8000;
        const SYMBOLIC: u32 = 0xA000;

        let stat = nix::sys::stat::fstat(self.file.as_raw_fd())?;
        match stat.st_mode & FILE_TYPE_BITS {
            CHARACTER => Ok(FileType::CharacterDevice),
            DIRECTORY => Ok(FileType::Directory),
            BLOCK => Ok(FileType::BlockDevice),
            REGULAR => Ok(FileType::RegularFile),
            SYMBOLIC => Ok(FileType::SymbolicLink),
            _ => Ok(FileType::Other),
        }
    }

    pub fn raw(&self) -> i32 {
        self.file.as_raw_fd()
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}
