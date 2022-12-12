mod inquiry;
mod mode_select;
mod mode_sense;
mod read_capacity;
mod send_diagnostic;
pub mod sense;
mod unmap;

use std::{borrow::BorrowMut, mem::size_of};

use crate::{result_data::ResultData, DataDirection};

pub use send_diagnostic::TestResult;

pub trait Command {
    type CommandBuffer;
    type DataBuffer;
    /// usually set it to the same as DataBuffer, but it can also be something like Box<DataBuffer>
    type DataBufferWrapper: BorrowMut<Self::DataBuffer>;
    type SenseBuffer;
    type ReturnType;

    fn get_direction(&self) -> DataDirection;
    fn get_command(&self) -> Self::CommandBuffer;
    fn get_data(&self) -> Self::DataBufferWrapper;
    fn get_sense_buffer(&self) -> Self::SenseBuffer;

    /// useful if have some custom data wrapper or want to trim data
    fn get_data_size(&self) -> u32 {
        size_of::<Self::DataBuffer>() as u32
    }

    fn process_result(
        &self,
        result: &ResultData<Self::DataBuffer, Self::SenseBuffer>,
    ) -> Self::ReturnType;
}
