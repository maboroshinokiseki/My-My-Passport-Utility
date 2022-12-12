use modular_bitfield_msb::prelude::*;

use crate::Scsi;

const LOGICAL_BLOCK_PROVISIONING_VPD_PAGE_CODE: u8 = 0xB2;

#[bitfield]
#[derive(Debug, Clone)]
pub struct LogicalBlockProvisioningVPDPage {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    threshold_exponent: B8,
    lbpu: B1,
    lbpws: B1,
    lbpws10: B1,
    lbprz: B3,
    anc_sup: B1,
    dp: B1,
    minimum_percentage: B5,
    provisioning_type: B3,
    threshold_percentage: B8,
    provisioning_group_descriptor1: B128,
    provisioning_group_descriptor2: B128,
    provisioning_group_descriptor3: B128,
    provisioning_group_descriptor4: B64,
}

impl Default for LogicalBlockProvisioningVPDPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Scsi {
    pub fn inquiry_unmap_support(&self) -> crate::Result<bool> {
        let data: LogicalBlockProvisioningVPDPage =
            self.inquiry_general(Some(LOGICAL_BLOCK_PROVISIONING_VPD_PAGE_CODE))?;

        Ok(data.lbpu() != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const LOGICAL_BLOCK_PROVISIONING_VPD_PAGE_LENGTH: usize = 64;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<LogicalBlockProvisioningVPDPage>(),
            LOGICAL_BLOCK_PROVISIONING_VPD_PAGE_LENGTH,
            concat!("Size of: ", stringify!(LogicalBlockProvisioningVPDPage))
        );
    }
}
