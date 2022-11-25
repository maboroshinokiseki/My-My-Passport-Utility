mod change_encryption_passphrase;
mod encryption_status;
mod read_handy_capacity;
mod read_handy_store;
mod reset_data_encryption_key;
mod unlock_encryption;
mod write_handy_store;

pub mod mode;

use std::io;

use libscsi::Scsi;
use read_handy_capacity::HandyCapacity;

pub use encryption_status::*;

use crate::Cipher;

struct WdVscWrapper {}

pub const HANDY_STORE_BLOCK_SIZE: usize = 512;
pub const DATA_SIGNATURE: u8 = 0x45;
pub const SALT_SIZE_FOR_U8: usize = 8;
pub const DEFAULT_SALT: [u8; SALT_SIZE_FOR_U8] = [87, 0, 68, 0, 67, 0, 46, 0];
pub const DEFAULT_ITERATION_COUNT: u32 = 1000;
pub const MAX_HINT_SIZE_FOR_U16: usize = 101;

pub trait WdVsc {
    fn encryption_status(&self) -> io::Result<EncryptionStatus>;
    fn read_handy_capacity(&self) -> io::Result<HandyCapacity>;
    fn read_handy_store(&self, index: u32) -> io::Result<[u8; HANDY_STORE_BLOCK_SIZE]>;
    fn write_handy_store(&self, index: u32, data: [u8; HANDY_STORE_BLOCK_SIZE]) -> io::Result<()>;
    fn unlock_encryption(&self, password: Vec<u8>) -> crate::Result<()>;
    fn change_encryption_passphrase(
        &self,
        cipher: Cipher,
        new_password: Option<Vec<u8>>,
        old_password: Option<Vec<u8>>,
    ) -> crate::Result<()>;

    fn reset_data_encryption_key(
        &self,
        cipher: Cipher,
        key_reset_enabler: u32,
    ) -> crate::Result<()>;
}

impl WdVsc for Scsi {
    fn encryption_status(&self) -> io::Result<EncryptionStatus> {
        WdVscWrapper::encryption_status(self)
    }

    fn read_handy_capacity(&self) -> io::Result<HandyCapacity> {
        WdVscWrapper::read_handy_capacity(self)
    }

    fn read_handy_store(&self, index: u32) -> io::Result<[u8; HANDY_STORE_BLOCK_SIZE]> {
        WdVscWrapper::read_handy_store(self, index)
    }

    fn write_handy_store(&self, index: u32, data: [u8; HANDY_STORE_BLOCK_SIZE]) -> io::Result<()> {
        WdVscWrapper::write_handy_store(self, index, data)
    }

    fn unlock_encryption(&self, password: Vec<u8>) -> crate::Result<()> {
        WdVscWrapper::unlock_encryption(self, password)
    }

    fn change_encryption_passphrase(
        &self,
        cipher: Cipher,
        new_password: Option<Vec<u8>>,
        old_password: Option<Vec<u8>>,
    ) -> crate::Result<()> {
        WdVscWrapper::change_encryption_passphrase(self, cipher, new_password, old_password)
    }

    fn reset_data_encryption_key(
        &self,
        cipher: Cipher,
        key_reset_enabler: u32,
    ) -> crate::Result<()> {
        WdVscWrapper::reset_data_encryption_key(self, cipher, key_reset_enabler)
    }
}
