#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

use super::sense::{BytesSenseBuffer, Sense};

const OPERATION_CODE: u8 = 0x55;

#[bitfield]
struct ModeSelectCommand {
    operation_code: B8,
    reserved_0: B3,
    page_format: B1,
    reserved_1: B3,
    saved_pages: B1,
    reserved_2: B40,
    parameter_list_length: B16,
    control: B8,
}

struct ThisCommand<T> {
    data: T,
}

impl<T> Command for ThisCommand<T>
where
    T: Copy,
{
    type CommandBuffer = ModeSelectCommand;

    type DataBuffer = T;

    type DataBufferWrapper = T;

    type SenseBuffer = BytesSenseBuffer;

    type ReturnType = crate::Result<()>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_page_format(1)
            .with_saved_pages(1)
            .with_parameter_list_length(self.get_data_size() as u16)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        self.data
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
}

impl Scsi {
    pub fn mode_select<T: Copy>(&self, data: T) -> crate::Result<()> {
        self.execute_command(&ThisCommand { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const MODE_SENSE_COMMAND_LENGTH: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ModeSelectCommand>(),
            MODE_SENSE_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(ModeSenseCommand))
        );
    }
}
