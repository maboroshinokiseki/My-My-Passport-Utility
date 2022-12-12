use modular_bitfield_msb::prelude::*;

pub trait Sense {
    fn default() -> Self;
    fn as_byte_slice(&self) -> &[u8];
}

pub type BytesSenseBuffer = [u8; 255];

impl Sense for BytesSenseBuffer {
    fn default() -> Self {
        [0; 255]
    }

    fn as_byte_slice(&self) -> &[u8] {
        &self[..]
    }
}

#[bitfield]
#[derive(Debug)]
pub struct FixedSenseBuffer {
    pub valid: B1,
    pub response_code: B7,
    pub obsolete: B8,
    pub filemark: B1,
    pub eom: B1,
    pub ili: B1,
    pub reserved_0: B1,
    pub sense_key: B4,
    pub information: B32,
    pub additional_sense_length: B8,
    pub command_specific_information: B32,
    pub additional_sense_code: B8,
    pub additional_sense_code_qualifier: B8,
    pub field_replaceable_unit_code: B8,
    pub sksv: B1,
    pub sense_key_specific: B23,
    pub additional_sense_bytes_0: B128,
}

impl Sense for FixedSenseBuffer {
    fn default() -> Self {
        Self::new()
    }

    fn as_byte_slice(&self) -> &[u8] {
        &self.bytes
    }
}
