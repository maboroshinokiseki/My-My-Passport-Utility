#![allow(dead_code)]

use std::io;

use modular_bitfield_msb::prelude::*;

use libscsi::Scsi;

const PAGE_CODE: u8 = 0x20;
const PAGE_LENGTH: u8 = 0x06;
const SIGNATURE: u8 = 0x30;

#[bitfield]
#[derive(Debug, Clone, Copy, Default)]
struct DeviceConfigurationPage {
    header: B64,
    parameter_savable: B1,
    reserved_0: B1,
    page_code: B6,
    page_length: B8,
    signature: B8,
    reserved_1: B8,
    /// Unknown
    disable_ap: B1,
    /// Unknown, but wd tool sets it to 0b01100
    unknown: B5,
    disable_cdrom: B1,
    disable_ses: B1,
    reserved_2: B6,
    two_tb_limit: B1,
    /// Unknown
    disable_white_list: B1,
    reserved_3: B16,
}

impl DeviceConfigurationPage {
    fn is_virtual_cd_on(&self) -> bool {
        self.disable_cdrom() == 0
    }
}

fn read(device: &Scsi) -> io::Result<DeviceConfigurationPage> {
    device.mode_sense::<DeviceConfigurationPage>(PAGE_CODE)
}

pub fn get_virtual_cd_status(device: &Scsi) -> io::Result<bool> {
    Ok(read(device)?.is_virtual_cd_on())
}

pub fn set_virtual_cd_status(device: &Scsi, enable_cd: bool) -> io::Result<()> {
    let data = read(device)?
        .with_header(0)
        .with_parameter_savable(0)
        .with_disable_cdrom(!enable_cd as u8);
    device.mode_select(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const DEVICE_CONFIGURATION_PAGE_SIZE: usize = 8 + 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<DeviceConfigurationPage>(),
            DEVICE_CONFIGURATION_PAGE_SIZE,
            concat!("Size of: ", stringify!(DeviceConfigurationPage))
        );
    }
}
