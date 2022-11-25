use std::io;

use crate::Scsi;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct StandardInquiryData {
    not_interested: [u8; 16],
    product_identification: [u8; 16],
}

impl Scsi {
    pub fn inquiry_product_identification(&self) -> io::Result<String> {
        let data: StandardInquiryData = self.inquiry_general(None)?;

        let product_identification = String::from_utf8_lossy(&data.product_identification);

        Ok(product_identification.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const STANDARD_INQUIRY_DATA_LENGTH: usize = 32;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<StandardInquiryData>(),
            STANDARD_INQUIRY_DATA_LENGTH,
            concat!("Size of: ", stringify!(StandardInquiryData))
        );
    }
}
