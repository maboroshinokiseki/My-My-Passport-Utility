#![allow(dead_code)]

use std::io;

use modular_bitfield_msb::prelude::*;

use super::{helper, FixedSenseBuffer};
use crate::{Command, DataDirection, Scsi, SgIoHeader};

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

    type ReturnType = TestResult;

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
        helper::fixed_sense_buffer_value()
    }

    fn process_result(
        &self,
        ioctl_result: i32,
        io_header: &SgIoHeader<Self::CommandBuffer, Self::DataBuffer, Self::SenseBuffer>,
    ) -> Self::ReturnType {
        if let Err(error) = helper::check_ioctl_result(ioctl_result) {
            return TestResult::Other(error);
        }

        let sense = io_header.sense_buffer.as_ref().unwrap();
        if sense.sense_key() == 0x04 {
            return TestResult::HardwareError;
        }
        let test_result = helper::check_error_status_any_sense(
            io_header.masked_status,
            io_header.host_status,
            io_header.driver_status,
            io_header.sense_buffer_written,
            Some(sense.as_slice()),
        );

        match test_result {
            Ok(_) => TestResult::Ok,
            Err(e) => TestResult::Other(e),
        }
    }
}

pub enum TestResult {
    Ok,
    HardwareError,
    Other(io::Error),
}

impl Scsi {
    pub fn send_diagnostic(&self) -> TestResult {
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
