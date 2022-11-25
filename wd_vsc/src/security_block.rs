use std::{io, mem::size_of, slice};

use libscsi::Scsi;

use crate::{WdVsc, MAX_HINT_SIZE_FOR_U16};

const SECURITY_BLOCK_INDEX: u32 = 1;
const SECURITY_BLOCK_SIGNATURE: [u8; 4] = [0u8, 1u8, b'D', b'W'];

#[derive(Debug, Clone)]
pub struct SecurityBlock {
    pub iteration_count: u32,
    pub salt: [u8; 8],
    pub hint: String,
}

#[repr(C)]
#[derive(Debug)]
struct SecurityBlockRaw {
    signature: [u8; 4],
    _reserved_0: [u8; 4],
    iteration_count: [u8; 4],
    salt: [u8; 8],
    _reserved_1: [u8; 4],
    hint: [u16; MAX_HINT_SIZE_FOR_U16],
    _reserved_2: [u8; 285],
    checksum: u8,
}

pub fn read_security_block(scsi: &Scsi) -> io::Result<SecurityBlock> {
    let raw_block = scsi.read_handy_store(SECURITY_BLOCK_INDEX)?;
    let (h, block, t) = unsafe { raw_block.as_slice().align_to::<SecurityBlockRaw>() };
    if !h.is_empty() || !t.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Invalid security block",
        ));
    }

    let block = &block[0];

    if block.signature != SECURITY_BLOCK_SIGNATURE {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Invalid security block, signature does not match",
        ));
    }

    let mut sum = 0u8;
    for n in raw_block {
        sum = sum.wrapping_add(n);
    }

    if sum != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Invalid security block, checksum error",
        ));
    }

    let iteration_count = u32::from_le_bytes(block.iteration_count);

    let terminator = block
        .hint
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(block.hint.len());
    let hint = String::from_utf16(&block.hint[..terminator])
        .unwrap_or_else(|e| {
            eprintln!("Invalid hint string, {}", e);
            String::new()
        })
        .trim()
        .to_owned();

    Ok(SecurityBlock {
        iteration_count,
        salt: block.salt,
        hint,
    })
}

pub fn write_security_block(
    scsi: &Scsi,
    iteration_count: u32,
    salt: [u8; 8],
    hint: String,
) -> io::Result<()> {
    let mut hint: Vec<u16> = hint.encode_utf16().collect();
    hint.push(0);
    if hint.len() > MAX_HINT_SIZE_FOR_U16 {
        return Err(io::Error::new(io::ErrorKind::Other, "hint is too long"));
    }
    let mut hint_block = [0u16; MAX_HINT_SIZE_FOR_U16];
    hint_block[..hint.len()].copy_from_slice(&hint);

    let mut block = SecurityBlockRaw {
        signature: SECURITY_BLOCK_SIGNATURE,
        _reserved_0: Default::default(),
        iteration_count: iteration_count.to_le_bytes(),
        salt,
        _reserved_1: Default::default(),
        hint: hint_block,
        _reserved_2: [0; 285],
        checksum: 0,
    };

    let block_slice: &[u8] = unsafe {
        slice::from_raw_parts(
            &block as *const _ as *const _,
            size_of::<SecurityBlockRaw>(),
        )
    };

    let mut sum = 0u8;
    for n in block_slice {
        sum = sum.wrapping_add(*n);
    }

    let checksum = 0u8.wrapping_sub(sum);
    block.checksum = checksum;

    scsi.write_handy_store(SECURITY_BLOCK_INDEX, block_slice.try_into().unwrap())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::HANDY_STORE_BLOCK_SIZE;

    use super::*;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<SecurityBlockRaw>(),
            HANDY_STORE_BLOCK_SIZE,
            concat!("Size of: ", stringify!(SecurityBlockRaw))
        );
    }
}
