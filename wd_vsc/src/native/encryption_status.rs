#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use libscsi::{
    command::{
        sense::{BytesSenseBuffer, Sense},
        *,
    },
    DataDirection, ResultData, Scsi,
};

use crate::{Cipher, SecurityStatus, DATA_SIGNATURE};

const OPERATION_CODE: u8 = 0xc0;
const OPERATION_SUBCODE: u8 = 0x45;
const TOTAL_DATA_SIZE: usize = 0x30;
const CIPHER_LIST_SIZE: usize = TOTAL_DATA_SIZE - size_of::<EncryptionStatusData>();

#[derive(Debug)]
pub struct EncryptionStatus {
    pub security_status: SecurityStatus,
    pub current_cipher: Cipher,
    pub blob_password_length: u16,
    pub key_reset_enabler: u32,
    pub supported_ciphers: Vec<Cipher>,
}

#[bitfield]
struct EncryptionStatusCommand {
    operation_code: B8,
    operation_subcode: B8,
    reserved: B40,
    allocation_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
struct EncryptionStatusData {
    signature: B8,
    reserved_0: B16,
    security_status: B8,
    current_cipher_id: B8,
    reserved_1: B8,
    password_length: B16,
    key_reset_enabler: B32,
    reserved_2: B24,
    number_of_ciphers: B8,
}

#[repr(packed)]
#[derive(Debug)]
struct EncryptionStatusDataWithCiphers {
    encryption_status_data: EncryptionStatusData,
    ciphers: [u8; CIPHER_LIST_SIZE],
}

struct ThisCommand {}

impl Command for ThisCommand {
    type CommandBuffer = EncryptionStatusCommand;

    type DataBuffer = EncryptionStatusDataWithCiphers;

    type DataBufferWrapper = EncryptionStatusDataWithCiphers;

    type SenseBuffer = BytesSenseBuffer;

    type ReturnType = crate::Result<EncryptionStatus>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_operation_subcode(OPERATION_SUBCODE)
            .with_allocation_length(size_of::<Self::DataBuffer>() as u16)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        Self::DataBufferWrapper {
            encryption_status_data: EncryptionStatusData::new(),
            ciphers: Default::default(),
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

        let result = result.data.as_ref().unwrap();
        let status = &result.encryption_status_data;
        if status.signature() != DATA_SIGNATURE {
            return Err(crate::Error::Other(
                "Invalid response signature, unsupported device.".to_owned(),
            ));
        }

        let security_status: SecurityStatus = status.security_status().try_into()?;

        let current_cipher: Cipher = status.current_cipher_id().into();

        let number_of_ciphers = status.number_of_ciphers() as usize;
        if number_of_ciphers > CIPHER_LIST_SIZE {
            eprintln!("There are too many supported ciphers.");
        }

        let number_of_ciphers = usize::min(number_of_ciphers, CIPHER_LIST_SIZE);

        let supported_ciphers = result
            .ciphers
            .iter()
            .take(number_of_ciphers)
            .map(|c| Cipher::from(*c))
            .collect();

        Ok(EncryptionStatus {
            security_status,
            current_cipher,
            blob_password_length: status.password_length(),
            key_reset_enabler: status.key_reset_enabler(),
            supported_ciphers,
        })
    }
}

impl super::WdVscWrapper {
    pub(super) fn encryption_status(scsi: &Scsi) -> crate::Result<EncryptionStatus> {
        scsi.execute_command(&ThisCommand {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const ENCRYPTION_STATUS_COMMAND_LENGTH: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<EncryptionStatusCommand>(),
            ENCRYPTION_STATUS_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(EncryptionStatusCommand))
        );

        assert_eq!(
            size_of::<EncryptionStatusDataWithCiphers>(),
            TOTAL_DATA_SIZE,
            concat!("Size of: ", stringify!(EncryptionStatusDataWithCiphers))
        );
    }
}
