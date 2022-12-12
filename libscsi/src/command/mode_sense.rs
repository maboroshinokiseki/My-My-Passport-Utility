#![allow(dead_code)]

use std::marker::PhantomData;

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

use super::sense::{BytesSenseBuffer, Sense};

const OPERATION_CODE: u8 = 0x5a;

#[bitfield]
struct ModeSenseCommand {
    operation_code: B8,
    reserved_0: B3,
    llbaa: B1,
    dbd: B1,
    reserved_1: B3,
    pc: B2,
    page_code: B6,
    subpage_code: B8,
    reserved_2: B24,
    allocation_length: B16,
    control: B8,
}

struct ThisCommand<T> {
    page_code: u8,
    phantom_data: PhantomData<T>,
}

impl<T> Command for ThisCommand<T>
where
    T: Default + Clone,
{
    type CommandBuffer = ModeSenseCommand;

    type DataBuffer = T;

    type DataBufferWrapper = T;

    type SenseBuffer = BytesSenseBuffer;

    type ReturnType = crate::Result<T>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_dbd(1)
            .with_page_code(self.page_code)
            .with_allocation_length(self.get_data_size() as u16)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {
        T::default()
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

        Ok(T::clone(result.data.as_ref().unwrap()))
    }
}

impl Scsi {
    pub fn mode_sense<T: Default + Clone>(&self, page_code: u8) -> crate::Result<T> {
        self.execute_command(&ThisCommand {
            page_code,
            phantom_data: PhantomData,
        })
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
            size_of::<ModeSenseCommand>(),
            MODE_SENSE_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(ModeSenseCommand))
        );
    }
}
