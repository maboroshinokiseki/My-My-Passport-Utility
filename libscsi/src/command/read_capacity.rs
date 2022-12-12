#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

use super::sense::{BytesSenseBuffer, Sense};

const OPERATION_CODE: u8 = 0x9e;
const READ_CAPACITY_16_SERVICE_ACTION: u8 = 0x10;

#[derive(Debug)]
pub struct Capacity {
    pub logical_block_count: u64,
    pub logical_block_length_in_bytes: u32,
}

#[bitfield]
struct ReadCapacity16Command {
    operation_code: B8,
    reserved1: B3,
    service_action: B5,
    obsolete_logical_block_address: u64,
    allocation_length: B32,
    reserved2: B7,
    obsolete_pmi: B1,
    controle: B8,
}

#[bitfield]
struct ReadCapacity16ParameterData {
    returned_logical_block_address: B64,
    logical_block_length_in_bytes: B32,
    reserved1: B2,
    rc_basis: B2,
    p_type: B3,
    prot_en: B1,
    p_i_exponent: B4,
    logical_blocks_per_physical_block_exponent: B4,
    lbpme: B1,
    lbprz: B1,
    lowest_aligned_logical_block_address: B14,
    reserved2: B128,
}

struct ThisCommand {}

impl Command for ThisCommand {
    type CommandBuffer = ReadCapacity16Command;

    type DataBuffer = ReadCapacity16ParameterData;

    type DataBufferWrapper = Self::DataBuffer;

    type SenseBuffer = BytesSenseBuffer;

    type ReturnType = crate::Result<Capacity>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        ReadCapacity16Command::new()
            .with_operation_code(OPERATION_CODE)
            .with_service_action(READ_CAPACITY_16_SERVICE_ACTION)
            .with_allocation_length(self.get_data_size())
    }

    fn get_data(&self) -> Self::DataBuffer {
        ReadCapacity16ParameterData::new()
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
        Ok(Capacity {
            logical_block_count: result.returned_logical_block_address() + 1,
            logical_block_length_in_bytes: result.logical_block_length_in_bytes(),
        })
    }
}

impl Scsi {
    pub fn read_capacity16(&self) -> crate::Result<Capacity> {
        let this_command = ThisCommand {};
        self.execute_command(&this_command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const SG_READ_CAPACITY16_CMD_LEN: usize = 16;
    const SG_READ_CAPACITY16_PARAMETER_LEN: usize = 32;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ReadCapacity16Command>(),
            SG_READ_CAPACITY16_CMD_LEN,
            concat!("Size of: ", stringify!(ReadCapacity16Command))
        );

        assert_eq!(
            size_of::<ReadCapacity16ParameterData>(),
            SG_READ_CAPACITY16_PARAMETER_LEN,
            concat!("Size of: ", stringify!(ReadCapacity16ParameterData))
        );
    }
}
