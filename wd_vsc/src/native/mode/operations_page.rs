#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use libscsi::Scsi;

const PAGE_CODE: u8 = 0x21;
const PAGE_LENGTH: u8 = 0x0a;
const SIGNATURE: u8 = 0x30;

#[bitfield]
#[derive(Debug, Clone, Copy, Default)]
struct OperationsPage {
    header: B64,
    parameter_savable: B1,
    reserved_0: B1,
    page_code: B6,
    page_length: B8,
    signature: B8,
    reserved_1: B8,
    reserved_2: B6,
    /// Unknown
    loose_sb2: B1,
    /// Unknown
    e_sata15: B1,
    reserved_3: B6,
    /// Unknown
    cd_media_valid: B1,
    /// Unknown
    enable_cd_eject: B1,
    reserved_4: B16,
    power_led_brite: B8,
    backlight_brite: B8,
    reserved_5: B7,
    inverted_lcd: B1,
    reserved_6: B8,
}

impl OperationsPage {
    fn get_led_brightness(&self) -> u8 {
        self.power_led_brite()
    }
}

fn read(device: &Scsi) -> crate::Result<OperationsPage> {
    Ok(device.mode_sense::<OperationsPage>(PAGE_CODE)?)
}

pub fn get_led_brightness(device: &Scsi) -> crate::Result<u8> {
    Ok(read(device)?.get_led_brightness())
}

pub fn set_led_brightness(device: &Scsi, led_brightness: u8) -> crate::Result<()> {
    let data = read(device)?
        .with_header(0)
        .with_parameter_savable(0)
        .with_power_led_brite(led_brightness);

    Ok(device.mode_select(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const OPERATIONS_PAGE_SIZE: usize = 8 + 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<OperationsPage>(),
            OPERATIONS_PAGE_SIZE,
            concat!("Size of: ", stringify!(OperationsPage))
        );
    }
}
