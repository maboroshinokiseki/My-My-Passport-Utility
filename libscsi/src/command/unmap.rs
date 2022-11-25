#![allow(dead_code)]

use std::{io, mem::size_of};

use modular_bitfield_msb::prelude::*;

use super::helper;
use crate::{Command, DataDirection, Scsi, SgIoHeader};

const OPERATION_CODE: u8 = 0x42;

#[bitfield]
struct UnmapCommand {
    operation_code: B8,
    reserved_0: B7,
    anchor: B1,
    reserved_1: B32,
    reserved_2: B3,
    group_number: B5,
    parameter_list_length: B16,
    control: B8,
}

#[repr(C, packed)]
struct UnmapParameterList<const N: usize> {
    /// size of unmap_block_descriptor_data_length + _reserved + unmap_block_descriptors
    unmap_data_length: u16,
    /// size of unmap_block_descriptors
    unmap_block_descriptor_data_length: u16,
    _reserved: u32,
    unmap_block_descriptors: [UnmapBlockDescriptor; N],
}

#[repr(C, packed)]
struct UnmapBlockDescriptor {
    unmap_logical_block_address: u64,
    number_of_logical_blocks: u32,
    _reserved: u32,
}

struct ThisCommand {
    unmap_logical_block_address: u64,
    number_of_logical_blocks: u32,
}

impl Command for ThisCommand {
    type CommandBuffer = UnmapCommand;

    type DataBuffer = UnmapParameterList<1>;

    type DataBufferWrapper = Self::DataBuffer;

    type SenseBuffer = helper::BytesSenseBuffer;

    type ReturnType = io::Result<()>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        UnmapCommand::new()
            .with_operation_code(OPERATION_CODE)
            .with_parameter_list_length(self.get_data_size() as u16)
    }

    fn get_data(&self) -> Self::DataBuffer {
        type T = UnmapParameterList<1>;
        T {
            unmap_data_length: ((size_of::<T>() - size_of::<u16>()) as u16).to_be(),
            unmap_block_descriptor_data_length: (size_of::<[UnmapBlockDescriptor; 1]>() as u16)
                .to_be(),
            _reserved: 0,
            unmap_block_descriptors: [UnmapBlockDescriptor {
                unmap_logical_block_address: self.unmap_logical_block_address.to_be(),
                number_of_logical_blocks: self.number_of_logical_blocks.to_be(),
                _reserved: 0,
            }],
        }
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
        helper::check_error_status(io_header)
    }
}

impl Scsi {
    pub fn unmap(&self, lba_offset: u64, lba_count: u64, max_unmap_lba: u32) -> io::Result<()> {
        // it's actually last_lba + 1
        let last_lba = lba_offset.saturating_add(lba_count);
        let lba_sets: Vec<_> = (lba_offset..last_lba - 1)
            .step_by(max_unmap_lba as usize)
            .into_iter()
            .map(|n| (n, u64::min(last_lba - n, max_unmap_lba as u64) as u32))
            .collect();

        for lba_set in lba_sets.iter() {
            let this_command = ThisCommand {
                unmap_logical_block_address: lba_set.0,
                number_of_logical_blocks: lba_set.1,
            };

            self.execute_command(&this_command)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const SG_UNMAP_CMD_LEN: usize = 10;
    const SG_UNMAP_PARAMETER_1_LEN: usize = 8 + 16;
    const SG_UNMAP_PARAMETER_2_LEN: usize = 8 + 16 * 2;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<UnmapCommand>(),
            SG_UNMAP_CMD_LEN,
            concat!("Size of: ", stringify!(UnmapCommand))
        );

        assert_eq!(
            size_of::<UnmapParameterList<1>>(),
            SG_UNMAP_PARAMETER_1_LEN,
            concat!("Size of: ", stringify!(UnmapParameterList<1>))
        );

        assert_eq!(
            size_of::<UnmapParameterList<2>>(),
            SG_UNMAP_PARAMETER_2_LEN,
            concat!("Size of: ", stringify!(UnmapParameterList<2>))
        );
    }
}
