#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use libscsi::{command::*, DataDirection, MaskedStatus, Scsi, SgIoHeader};

use crate::{Error, Result, DATA_SIGNATURE};

const OPERATION_CODE: u8 = 0xc1;
const OPERATION_SUBCODE: u8 = 0xe1;
const MAX_PASSWORD_LENGTH: usize = 32;
const DATA_SIZE_WITHOUT_PASSWORD: usize = 8;

#[bitfield]
struct UnlockEncryptionCommand {
    operation_code: B8,
    operation_subcode: B8,
    reserved_0: B40,
    parameter_list_length: B16,
    control: B8,
}

struct UnlockEncryptionData {
    signature: u8,
    reserved_0: [u8; 5],
    password_length: [u8; 2],
    password: [u8; MAX_PASSWORD_LENGTH],
}

struct ThisCommand {
    password: Vec<u8>,
}

impl Command for ThisCommand {
    type CommandBuffer = UnlockEncryptionCommand;

    type DataBuffer = UnlockEncryptionData;

    type DataBufferWrapper = UnlockEncryptionData;

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
        let mut password = [0u8; MAX_PASSWORD_LENGTH];
        let password_length = self.password.len();
        password[..password_length].copy_from_slice(&self.password);
        UnlockEncryptionData {
            signature: DATA_SIGNATURE,
            reserved_0: [0; 5],
            password_length: (password_length as u16).to_be_bytes(),
            password,
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
                    if sens_buffer.additional_sense_code_qualifier() == 0x81 {
                        return Err(Error::NotLocked("Device is unlocked already.".to_owned()));
                    } else if sens_buffer.additional_sense_code_qualifier() == 0x80 {
                        return Err(Error::ExceedUnlockAttempts);
                    } else if sens_buffer.additional_sense_code_qualifier() == 0x40 {
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
        (DATA_SIZE_WITHOUT_PASSWORD + self.password.len()) as u32
    }
}

impl super::WdVscWrapper {
    pub(super) fn unlock_encryption(scsi: &Scsi, password: Vec<u8>) -> Result<()> {
        if password.len() > MAX_PASSWORD_LENGTH {
            return Err(Error::PasswordBlobSizeIncorrect);
        }

        scsi.execute_command(&ThisCommand { password })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const UNLOCK_ENCRYPTION_COMMAND_LENGTH: usize = 10;
    const UNLOCK_ENCRYPTION_DATA_LENGTH: usize = 40;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<UnlockEncryptionCommand>(),
            UNLOCK_ENCRYPTION_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(UnlockEncryptionCommand))
        );

        assert_eq!(
            size_of::<UnlockEncryptionData>(),
            UNLOCK_ENCRYPTION_DATA_LENGTH,
            concat!("Size of: ", stringify!(UnlockEncryptionData))
        );
    }
}
