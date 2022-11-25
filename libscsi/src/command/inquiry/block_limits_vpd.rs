use std::io;

use modular_bitfield_msb::prelude::*;

use crate::Scsi;

const BLOCK_LIMITS_VPD_PAGE_CODE: u8 = 0xb0;

#[bitfield]
#[derive(Debug, Clone)]
pub struct BlockLimitsVPDPage {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    reserved: B7,
    wsnz: B1,
    maximum_compare_and_write_length: B8,
    optimal_transfer_length_granularity: B16,
    maximum_transfer_length: B32,
    optimal_transfer_length: B32,
    maximum_prefetch_length: B32,
    maximum_unmap_lba_count: B32,
    maximum_unmap_block_descriptor_count: B32,
    optimal_unmap_granularity: B32,
    ugavalid: B1,
    unmap_granularity_alignment: B31,
    maximum_write_same_length: B64,
    maximum_atomic_transfer_length: B32,
    atomic_alignment: B32,
    atomic_transfer_length_granularity: B32,
    maximum_atomic_transfer_length_with_atomic_boundary: B32,
    maximum_atomic_boundary_size: B32,
}

impl Default for BlockLimitsVPDPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Scsi {
    pub fn inquiry_unmap_block_limit(&self) -> io::Result<u32> {
        let data: BlockLimitsVPDPage = self.inquiry_general(Some(BLOCK_LIMITS_VPD_PAGE_CODE))?;

        Ok(data.maximum_unmap_lba_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const BLOCK_LIMITS_VPD_PAGE_LENGTH: usize = 64;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<BlockLimitsVPDPage>(),
            BLOCK_LIMITS_VPD_PAGE_LENGTH,
            concat!("Size of: ", stringify!(BlockLimitsVPDPage))
        );
    }
}
