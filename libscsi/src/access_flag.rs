use std::ffi::c_uint;

use bitflags::bitflags;

bitflags! {
    pub struct AccessFlags: c_uint {
        /// Indirect io
        const DEFAULT           = 0b0000;
        const DIRECT_IO         = 0b0001;
        const LUN_INHIBIT       = 0b0010;
        const MEMORY_MAPPED_IO  = 0b0100;
        const NO_DATA_TRANSFER  = 0b1000;
    }
}
