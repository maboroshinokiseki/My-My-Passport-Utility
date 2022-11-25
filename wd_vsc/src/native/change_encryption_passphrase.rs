#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use libscsi::{command::*, DataDirection, MaskedStatus, Scsi, SgIoHeader};

use crate::{Cipher, Error, Result, DATA_SIGNATURE};

const OPERATION_CODE: u8 = 0xc1;
const OPERATION_SUBCODE: u8 = 0xe2;
const MAX_PASSWORD_LENGTH: usize = 32;
const DATA_SIZE_WITHOUT_PASSWORD: usize = 8;
const USE_VENDOR_OLD_PASSWORD: u8 = 0b00000001;
const USE_VENDOR_NEW_PASSWORD: u8 = 0b00010000;

#[bitfield]
struct ChangeEncryptionPassphraseCommand {
    operation_code: B8,
    operation_subcode: B8,
    reserved_0: B40,
    parameter_list_length: B16,
    control: B8,
}

struct ChangeEncryptionPassphraseData {
    signature: u8,
    reserved_0: [u8; 2],
    use_vendor_password: u8,
    reserved_1: [u8; 2],
    password_length: [u8; 2],
    passwords: [u8; MAX_PASSWORD_LENGTH * 2],
}

struct ThisCommand {
    new_password: Option<Vec<u8>>,
    old_password: Option<Vec<u8>>,
    password_length: usize,
}

impl Command for ThisCommand {
    type CommandBuffer = ChangeEncryptionPassphraseCommand;

    type DataBuffer = ChangeEncryptionPassphraseData;

    type DataBufferWrapper = ChangeEncryptionPassphraseData;

    type SenseBuffer = FixedSenseBuffer;

    type ReturnType = Result<()>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_operation_subcode(OPERATION_SUBCODE)
            .with_parameter_list_length(self.get_data_size() as u16)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        let mut use_vendor_password = 0;
        let mut passwords = [0u8; MAX_PASSWORD_LENGTH * 2];
        match self.old_password.as_ref() {
            Some(p) => passwords[..self.password_length].copy_from_slice(p),
            None => use_vendor_password = USE_VENDOR_OLD_PASSWORD,
        }

        match self.new_password.as_ref() {
            Some(p) => passwords[self.password_length..self.password_length * 2].copy_from_slice(p),
            None => use_vendor_password = USE_VENDOR_NEW_PASSWORD,
        }

        ChangeEncryptionPassphraseData {
            signature: DATA_SIGNATURE,
            reserved_0: Default::default(),
            use_vendor_password,
            reserved_1: Default::default(),
            password_length: (self.password_length as u16).to_be_bytes(),
            passwords,
        }
    }

    fn get_sense_buffer(&self) -> Self::SenseBuffer {
        helper::fixed_sense_buffer_value()
    }

    fn process_result(
        &self,
        ioctl_result: i32,
        io_header: &SgIoHeader<Self::CommandBuffer, Self::DataBuffer, Self::SenseBuffer>,
    ) -> Self::ReturnType {
        helper::check_ioctl_result(ioctl_result)?;

        if io_header.masked_status == MaskedStatus::CHECK_CONDITION {
            let sens_buffer = io_header.sense_buffer.as_ref().unwrap();
            // ILLEGAL REQUEST
            if sens_buffer.sense_key() == 0x05 {
                if sens_buffer.additional_sense_code() == 0x74 {
                    if sens_buffer.additional_sense_code_qualifier() == 0x40 {
                        return Err(Error::PasswordIncorrect);
                    }
                } else if (sens_buffer.additional_sense_code() == 0x24
                    || sens_buffer.additional_sense_code() == 0x26)
                    && sens_buffer.additional_sense_code_qualifier() == 0x00
                {
                    return Err(Error::PasswordBlobSizeIncorrect);
                }
            }
        }

        helper::check_error_status_any_sense(
            io_header.masked_status,
            io_header.host_status,
            io_header.driver_status,
            io_header.sense_buffer_written,
            Some(io_header.sense_buffer.as_ref().unwrap().as_slice()),
        )?;

        Ok(())
    }

    fn get_data_size(&self) -> u32 {
        (DATA_SIZE_WITHOUT_PASSWORD + self.password_length * 2) as u32
    }
}

impl super::WdVscWrapper {
    pub(super) fn change_encryption_passphrase(
        scsi: &Scsi,
        cipher: Cipher,
        new_password: Option<Vec<u8>>,
        old_password: Option<Vec<u8>>,
    ) -> Result<()> {
        let password_length = cipher.get_password_blob_size()?;
        let mut empty_count = 0;
        match new_password.as_ref() {
            Some(p) => {
                if p.len() != password_length {
                    return Err(Error::PasswordBlobSizeIncorrect);
                }
            }
            None => empty_count += 1,
        }

        match old_password.as_ref() {
            Some(p) => {
                if p.len() != password_length {
                    return Err(Error::PasswordBlobSizeIncorrect);
                }
            }
            None => empty_count += 1,
        }

        if empty_count == 2 {
            return Err(Error::Other("Both passwords are empty".to_owned()));
        }

        scsi.execute_command(&ThisCommand {
            new_password,
            old_password,
            password_length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const CHANGE_ENCRYPTION_PASSPHRASE_COMMAND_LENGTH: usize = 10;
    const CHANGE_ENCRYPTION_PASSPHRASE_DATA_LENGTH: usize =
        DATA_SIZE_WITHOUT_PASSWORD + MAX_PASSWORD_LENGTH * 2;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ChangeEncryptionPassphraseCommand>(),
            CHANGE_ENCRYPTION_PASSPHRASE_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(ChangeEncryptionPassphraseCommand))
        );

        assert_eq!(
            size_of::<ChangeEncryptionPassphraseData>(),
            CHANGE_ENCRYPTION_PASSPHRASE_DATA_LENGTH,
            concat!("Size of: ", stringify!(ChangeEncryptionPassphraseData))
        );
    }
}
