#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

use super::sense::{FixedSenseBuffer, Sense};

const OPERATION_CODE: u8 = 0x1d;

#[bitfield]
struct SendDiagnosticCommand {
    operation_code: B8,
    self_test_code: B3,
    pf: B1,
    reserved_0: B1,
    selftest: B1,
    devoffl: B1,
    unitoffl: B1,
    reserved_1: B8,
    parameter_list_length: B16,
    control: B8,
}

struct ThisCommand {}

impl Command for ThisCommand {
    type CommandBuffer = SendDiagnosticCommand;

    type DataBuffer = ();

    type DataBufferWrapper = ();

    type SenseBuffer = FixedSenseBuffer;

    type ReturnType = crate::Result<TestResult>;

    fn get_direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn get_command(&self) -> Self::CommandBuffer {
        Self::CommandBuffer::new()
            .with_operation_code(OPERATION_CODE)
            .with_selftest(1)
    }

    fn get_data(&self) -> Self::DataBufferWrapper {}

    fn get_sense_buffer(&self) -> Self::SenseBuffer {
        Self::SenseBuffer::default()
    }

    fn process_result(
        &self,
        result: &ResultData<Self::DataBuffer, Self::SenseBuffer>,
    ) -> Self::ReturnType {
        result.check_ioctl_error()?;
        let sense = result.sense_buffer.as_ref().unwrap();
        if sense.sense_key() == 0x04 {
            return Ok(TestResult::HardwareError);
        }

        result.check_common_error()?;

        Ok(TestResult::Ok)
    }
}

pub enum TestResult {
    Ok,
    HardwareError,
}

impl Scsi {
    pub fn send_diagnostic(&self) -> crate::Result<TestResult> {
        self.execute_command(&ThisCommand {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const SEND_DIAGNOSTIC_COMMAND_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<SendDiagnosticCommand>(),
            SEND_DIAGNOSTIC_COMMAND_LENGTH,
            concat!("Size of: ", stringify!(SendDiagnosticCommand))
        );
    }
}
