#![allow(dead_code)]

use std::io;

use modular_bitfield_msb::prelude::*;

use libscsi::Scsi;

const PAGE_CODE: u8 = 0x1a;
const PAGE_LENGTH: u8 = 0x26;

#[bitfield]
#[derive(Debug, Clone, Copy)]
struct PowerConditionModePage {
    /// header (start)
    mode_data_length: B16,
    medium_type: B8,
    wp: B1,
    reserved_0: B2,
    dpofua: B1,
    reserved_1: B4,
    reserved: B7,
    longlba: B1,
    reserved_2: B8,
    block_descriptor_length: B16,
    /// header (end)
    parameter_savable: B1,
    spf: B1,
    page_code: B6,
    page_length: B8,
    pm_bg_precedence: B2,
    reserved_3: B5,
    standby_y: B1,
    reserved_4: B4,
    idle_c: B1,
    idle_b: B1,
    idle_a: B1,
    standby_z: B1,
    idle_a_condition_timer: B32,
    standby_z_condition_timer: B32,
    idle_b_condition_timer: B32,
    idle_c_condition_timer: B32,
    standby_y_condition_timer: B32,
    reserved_5: B120,
    ccf_idle: B2,
    ccf_standby: B2,
    ccf_stopped: B2,
    reserved_6: B2,
}

impl Default for PowerConditionModePage {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerConditionModePage {
    fn get_sleep_timer(&self) -> u32 {
        self.standby_z_condition_timer() / 10
    }
}

fn read(device: &Scsi) -> io::Result<PowerConditionModePage> {
    device.mode_sense::<PowerConditionModePage>(PAGE_CODE)
}

pub fn get_sleep_timer(device: &Scsi) -> io::Result<u32> {
    Ok(read(device)?.get_sleep_timer())
}

pub fn set_sleep_timer(device: &Scsi, sleep_timer: u32) -> io::Result<()> {
    let (enable_timer, sleep_timer) = if sleep_timer == 0 {
        (false, 0)
    } else {
        // not sure if all wd my passport drives have this range limit
        (true, sleep_timer.clamp(60, 28800))
    };

    let data = PowerConditionModePage::new()
        .with_page_code(PAGE_CODE)
        .with_page_length(PAGE_LENGTH)
        .with_standby_z(enable_timer as u8)
        .with_standby_z_condition_timer(sleep_timer.saturating_mul(10));
    device.mode_select(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const POWER_CONDITION_MODE_PAGE_SIZE: usize = 8 + 40;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PowerConditionModePage>(),
            POWER_CONDITION_MODE_PAGE_SIZE,
            concat!("Size of: ", stringify!(PowerConditionModePage))
        );
    }
}
