#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use libscsi::{
    command::{
        sense::{BytesSenseBuffer, Sense},
        *,
    },
    DataDirection, ResultData, Scsi,
};

use crate::HANDY_STORE_BLOCK_SIZE;

const OPERATION_CODE: u8 = 0xd8;

#[bitfield]
struct ReadHandyStoreCommand {
    operation_code: B8,
    reserved_0: B8,
    handy_store_block_address: B32,
    reserved_1: B8,
    /// Number of handy store blocks
    transfer_length: B16,
    control: B8,
}

struct ThisCommand {
    handy_store_index: u32,
}

impl Command for ThisCommand {
    type CommandBuffer = ReadHandyStoreCommand;

    type DataBuffer = [u8; HANDY_STORE_BLOCK_SIZE];

    type DataBufferWrapper = [u8; HANDY_STORE_BLOCK_SIZE];

    type SenseBuffer = BytesSenseBuffer;

    type ReturnType = crate::Result<[u8; HANDY_STORE_BLOCK_SIZE]>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_handy_store_block_address(self.handy_store_index)
            .with_transfer_length(1)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        [0; HANDY_STORE_BLOCK_SIZE]
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

        Ok(**result.data.as_ref().unwrap())
    }
}

impl super::WdVscWrapper {
    pub(super) fn read_handy_store(
        scsi: &Scsi,
        index: u32,
    ) -> crate::Result<[u8; HANDY_STORE_BLOCK_SIZE]> {
        scsi.execute_command(&ThisCommand {
            handy_store_index: index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const READ_HANDY_STORE_COMMAND_LENGTH: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ReadHandyStoreCommand>(),
            READ_HANDY_STORE_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(ReadHandyStoreCommand))
        );
    }
}
