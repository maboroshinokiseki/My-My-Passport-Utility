use std::ffi::c_ushort;

use bitflags::bitflags;

bitflags! {
    pub struct DriverStatus: c_ushort {
        const OK            = 0x00;
        const BUSY          = 0x01;
        const SOFT          = 0x02;
        const MEDIA         = 0x03;
        const ERROR         = 0x04;
        const INVALID       = 0x05;
        const TIMEOUT       = 0x06;
        const HARD          = 0x07;
        /// Implies sense_buffer output
        const SENSE         = 0x08;
        // above status 'or'ed with one of the following suggestions
        const RETRY         = 0x10;
        const ABORT         = 0x20;
        const REMAP         = 0x30;
        const DIE           = 0x40;
        const SUGGEST_SENSE = 0x80;
    }
}
