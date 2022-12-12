use std::ffi::c_uint;

use bitflags::bitflags;

bitflags! {
    pub struct AuxiliaryInfo: c_uint {
        const OK_MASK           = 0x01;
        const OK                = 0x00;
        const CHECK             = 0x01;
        const DIRECT_IO_MASK    = 0x06;
        const INDIRECT_IO       = 0x00;
        const DIRECT_IO         = 0x02;
        const MIXED_IO          = 0x04;
    }
}
