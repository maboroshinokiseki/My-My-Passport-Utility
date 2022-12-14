#![allow(dead_code)]

use std::marker::PhantomData;

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

use super::sense::{BytesSenseBuffer, Sense};

mod block_limits_vpd;
mod logical_block_provisioning_vpd;
mod product_identification;

const OPERATION_CODE: u8 = 0x12;

#[bitfield]
struct InquiryCommand {
    operation_code: B8,
    reserved: B6,
    cmddt: B1,
    evpd: B1,
    page_code: B8,
    allocation_length: B16,
    control: B8,
}

struct ThisCommand<T> {
    enable_vpd: bool,
    page_code: u8,

    phantom_data: PhantomData<T>,
}

impl<T> Command for ThisCommand<T>
where
    T: Default + Clone,
{
    type CommandBuffer = InquiryCommand;

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
            .with_allocation_length(self.get_data_size() as u16)
            .with_evpd(self.enable_vpd as u8)
            .with_page_code(self.page_code)
    }

    fn get_data(&self) -> Self::DataBuffer {
        Self::DataBuffer::default()
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

        Ok(Clone::clone(result.data.as_ref().unwrap()))
    }
}

impl Scsi {
    fn inquiry_general<T: Default + Clone>(&self, page_code: Option<u8>) -> crate::Result<T> {
        let this_command = ThisCommand {
            enable_vpd: page_code.is_some(),
            page_code: page_code.unwrap_or_default(),
            phantom_data: PhantomData,
        };

        self.execute_command(&this_command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const INQUIRY_COMMAND_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<InquiryCommand>(),
            INQUIRY_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(InquiryCommand))
        );
    }
}
