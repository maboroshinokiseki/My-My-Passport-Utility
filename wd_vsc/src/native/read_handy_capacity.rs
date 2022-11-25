#![allow(dead_code)]

use std::io;

use modular_bitfield_msb::prelude::*;

use libscsi::{command::*, DataDirection, Scsi, SgIoHeader};

const OPERATION_CODE: u8 = 0xd5;

#[derive(Debug)]
pub struct HandyCapacity {
    pub last_handy_block_address: u32,
    pub block_length: u32,
    pub maximum_transfer_length: u16,
}

#[bitfield]
struct ReadHandyCapacityCommand {
    operation_code: B8,
    reserved: B64,
    control: B8,
}

#[bitfield]
#[derive(Debug)]
struct ReadHandyCapacityData {
    last_handy_block_address: B32,
    block_length: B32,
    reserved: B16,
    maximum_transfer_length: B16,
}

struct ThisCommand {}

impl Command for ThisCommand {
    type CommandBuffer = ReadHandyCapacityCommand;

    type DataBuffer = ReadHandyCapacityData;

    type DataBufferWrapper = ReadHandyCapacityData;

    type SenseBuffer = helper::BytesSenseBuffer;

    type ReturnType = io::Result<HandyCapacity>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new().with_operation_code(OPERATION_CODE)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        Self::DataBufferWrapper::new()
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

        let result = io_header.data.as_ref().unwrap();

        Ok(HandyCapacity {
            last_handy_block_address: result.last_handy_block_address(),
            block_length: result.block_length(),
            maximum_transfer_length: result.maximum_transfer_length(),
        })
    }
}

impl super::WdVscWrapper {
    pub(super) fn read_handy_capacity(scsi: &Scsi) -> io::Result<HandyCapacity> {
        scsi.execute_command(&ThisCommand {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const READ_HANDY_CAPACITY_COMMAND_LENGTH: usize = 10;
    const READ_HANDY_CAPACITY_DATA_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ReadHandyCapacityCommand>(),
            READ_HANDY_CAPACITY_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(ReadHandyCapacityCommand))
        );

        assert_eq!(
            size_of::<ReadHandyCapacityData>(),
            READ_HANDY_CAPACITY_DATA_LENGTH,
            concat!("Size of: ", stringify!(ReadHandyCapacityData))
        );
    }
}
