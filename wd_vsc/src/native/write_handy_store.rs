#![allow(dead_code)]

use std::io;

use modular_bitfield_msb::prelude::*;

use libscsi::{command::*, DataDirection, Scsi, SgIoHeader};

use crate::HANDY_STORE_BLOCK_SIZE;

const OPERATION_CODE: u8 = 0xda;

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
    data: [u8; HANDY_STORE_BLOCK_SIZE],
}

impl Command for ThisCommand {
    type CommandBuffer = ReadHandyStoreCommand;

    type DataBuffer = [u8; HANDY_STORE_BLOCK_SIZE];

    type DataBufferWrapper = [u8; HANDY_STORE_BLOCK_SIZE];

    type SenseBuffer = helper::BytesSenseBuffer;

    type ReturnType = io::Result<()>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_handy_store_block_address(self.handy_store_index)
            .with_transfer_length(1)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        self.data
    }

    fn get_sense_buffer(&self) -> Self::SenseBuffer {
        helper::bytes_sense_buffer_value()
    }

    fn process_result(
        &self,
        ioctl_result: i32,
        io_header: &SgIoHeader<Self::CommandBuffer, Self::DataBuffer, Self::SenseBuffer>,
    ) -> Self::ReturnType {
        helper::check_ioctl_result(ioctl_result)?;
        helper::check_error_status(io_header)?;

        Ok(())
    }
}

impl super::WdVscWrapper {
    pub(super) fn write_handy_store(
        scsi: &Scsi,
        index: u32,
        data: [u8; HANDY_STORE_BLOCK_SIZE],
    ) -> io::Result<()> {
        scsi.execute_command(&ThisCommand {
            handy_store_index: index,
            data,
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
