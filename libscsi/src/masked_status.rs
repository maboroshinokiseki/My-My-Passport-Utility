use std::ffi::c_uchar;

use bitflags::bitflags;

bitflags! {
    pub struct MaskedStatus: c_uchar {
        const GOOD                  = 0x00;
        const CHECK_CONDITION       = 0x01;
        const CONDITION_GOOD        = 0x02;
        const BUSY                  = 0x04;
        const INTERMEDIATE_GOOD     = 0x08;
        const INTERMEDIATE_C_GOOD   = 0x0a;
        const RESERVATION_CONFLICT  = 0x0c;
        const COMMAND_TERMINATED    = 0x11;
        const QUEUE_FULL            = 0x14;
    }
}
