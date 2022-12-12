#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;
use rand::{thread_rng, Rng};

use libscsi::{
    command::{
        sense::{BytesSenseBuffer, Sense},
        *,
    },
    DataDirection, ResultData, Scsi,
};

use crate::{Cipher, DATA_SIGNATURE};

const OPERATION_CODE: u8 = 0xc1;
const OPERATION_SUBCODE: u8 = 0xe3;
const MAX_PASSWORD_LENGTH: usize = 32;
const DATA_SIZE_WITHOUT_PASSWORD: usize = 8;

#[bitfield]
struct ResetDataEncryptionKeyCommand {
    operation_code: B8,
    operation_subcode: B8,
    key_reset_enabler: B32,
    reserved: B8,
    parameter_list_length: B16,
    control: B8,
}

struct ResetDataEncryptionKeyData {
    signature: u8,
    reserved_0: [u8; 2],
    /// Indicate if the key provided will be XORed with internal key.
    /// FullDiscEncryption doesn't support this setting.
    combine: u8,
    cipher_id: u8,
    reserved_1: u8,
    /// In bits.
    key_length: [u8; 2],
    /// FullDiscEncryption doesn't need a key.
    key: [u8; MAX_PASSWORD_LENGTH],
}

struct ThisCommand {
    cipher_id: u8,
    key_length_in_bytes: usize,
    key_reset_enabler: u32,
}

impl Command for ThisCommand {
    type CommandBuffer = ResetDataEncryptionKeyCommand;

    type DataBuffer = ResetDataEncryptionKeyData;

    type DataBufferWrapper = ResetDataEncryptionKeyData;

    type SenseBuffer = BytesSenseBuffer;

    type ReturnType = crate::Result<()>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_operation_subcode(OPERATION_SUBCODE)
            .with_key_reset_enabler(self.key_reset_enabler)
            .with_parameter_list_length(self.get_data_size() as u16)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        let mut key = [0u8; MAX_PASSWORD_LENGTH];
        thread_rng().fill(&mut key[..]);

        ResetDataEncryptionKeyData {
            signature: DATA_SIGNATURE,
            reserved_0: Default::default(),
            combine: 0,
            cipher_id: self.cipher_id,
            reserved_1: 0,
            key_length: ((self.key_length_in_bytes * 8) as u16).to_be_bytes(),
            key,
        }
    }

    fn get_sense_buffer(&self) -> Self::SenseBuffer {
        Self::SenseBuffer::default()
    }

    fn process_result(
        &self,
        result: &ResultData<Self::DataBuffer, Self::SenseBuffer>,
    ) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(())
    }

    fn get_data_size(&self) -> u32 {
        (DATA_SIZE_WITHOUT_PASSWORD + self.key_length_in_bytes) as u32
    }
}

impl super::WdVscWrapper {
    pub(super) fn reset_data_encryption_key(
        scsi: &Scsi,
        cipher: Cipher,
        key_reset_enabler: u32,
    ) -> crate::Result<()> {
        let password_length = match cipher {
            Cipher::FullDiscEncryption | Cipher::NoEncryption => 0,
            _ => cipher.get_password_blob_size()?,
        };

        scsi.execute_command(&ThisCommand {
            cipher_id: u8::from(cipher),
            key_length_in_bytes: password_length,
            key_reset_enabler,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const RESET_DATA_ENCRYPTION_KEY_COMMAND_LENGTH: usize = 10;
    const RESET_DATA_ENCRYPTION_KEY_DATA_LENGTH: usize =
        DATA_SIZE_WITHOUT_PASSWORD + MAX_PASSWORD_LENGTH;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ResetDataEncryptionKeyCommand>(),
            RESET_DATA_ENCRYPTION_KEY_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(ResetDataEncryptionKeyCommand))
        );

        assert_eq!(
            size_of::<ResetDataEncryptionKeyData>(),
            RESET_DATA_ENCRYPTION_KEY_DATA_LENGTH,
            concat!("Size of: ", stringify!(ResetDataEncryptionKeyData))
        );
    }
}
