use libscsi::Scsi;
use sha2::{Digest, Sha256};

use crate::{
    security_block::{read_security_block, write_security_block, SecurityBlock},
    Cipher, EncryptionStatus, Error, Result, SecurityStatus, WdVsc, DEFAULT_ITERATION_COUNT,
    DEFAULT_SALT, SALT_SIZE_FOR_U8,
};

/// return: SecurityBlock, IsFromDisk
pub fn read_security_block_or_default(device: &Scsi) -> (SecurityBlock, bool) {
    match read_security_block(device) {
        Ok(b) => (b, true),
        Err(_) => (
            SecurityBlock {
                iteration_count: DEFAULT_ITERATION_COUNT,
                salt: DEFAULT_SALT,
                hint: "".to_owned(),
            },
            false,
        ),
    }
}

pub fn create_salt_blob(salt: Option<String>) -> Result<Option<[u8; SALT_SIZE_FOR_U8]>> {
    let salt = match salt {
        Some(s) => {
            let mut salt = str_to_utf16_bytes(&s)?;
            if salt.len() > SALT_SIZE_FOR_U8 {
                return Err(Error::Other("Salt length is too long".to_owned()));
            }

            salt.resize(SALT_SIZE_FOR_U8, 0);

            Some(salt.try_into().unwrap())
        }
        None => None,
    };

    Ok(salt)
}

fn str_to_utf16_bytes(s: &str) -> Result<Vec<u8>> {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    let (h, bytes, t) = unsafe { utf16.as_slice().align_to::<u8>() };
    if !h.is_empty() || !t.is_empty() {
        return Err(Error::Other(format!("Failed to serialize string: {}", s)));
    }

    Ok(bytes.into())
}

pub fn create_password_blob(
    cipher: Cipher,
    salt: &[u8],
    iteration_count: u32,
    password: &str,
) -> Result<Vec<u8>> {
    let password_length = cipher.get_password_blob_size()?;

    let utf16: Vec<u16> = password.encode_utf16().collect();
    let (h, bytes, t) = unsafe { utf16.as_slice().align_to::<u8>() };
    if !h.is_empty() || !t.is_empty() {
        return Err(Error::Other("Failed to serialize password".to_owned()));
    }

    let mut blob = [salt, bytes].concat();
    for _ in 0..iteration_count {
        let mut hash = Sha256::new();
        hash.update(&blob);
        let result = hash.finalize();
        blob.clear();
        blob.extend_from_slice(&result[..]);
    }

    blob.truncate(password_length);

    Ok(blob)
}

#[allow(clippy::too_many_arguments)]
pub fn change_password(
    device: &Scsi,
    status: &EncryptionStatus,
    new_password: Option<String>,
    old_password: Option<String>,
    hint: Option<String>,
    new_iteration_count: Option<u32>,
    new_salt: Option<[u8; SALT_SIZE_FOR_U8]>,
    old_iteration_count: Option<u32>,
    old_salt: Option<[u8; SALT_SIZE_FOR_U8]>,
) -> Result<()> {
    match status.security_status {
        SecurityStatus::NoUserPassword => {}
        SecurityStatus::Locked => {
            return Err(Error::NotUnlocked(
                "In order to change password, the device has to be unlocked first.".to_owned(),
            ))
        }
        SecurityStatus::Unlocked => {}
        SecurityStatus::UnlockAttemptExceeded => return Err(Error::ExceedUnlockAttempts),
        SecurityStatus::NoEncryption => {}
    }

    let (new_salt,new_iteration_count )= unwrap_salt_and_iteration_count(
        Some(device),
        new_salt,
        new_iteration_count,
        Some("Warring: Failed to read salt from disk, will use the default value as new salt."),
        Some(
            "Warring: Failed to read iteration count from disk, will use the default value as new salt.",
        ),
    );

    let new_password_blob = match new_password {
        Some(p) => Some(create_password_blob(
            status.current_cipher,
            &new_salt,
            new_iteration_count,
            &p,
        )?),
        None => None,
    };

    let (old_salt,old_iteration_count )= unwrap_salt_and_iteration_count(
        Some(device),
        old_salt,
        old_iteration_count,
        Some("Warring: Failed to read salt from disk, will use the default value as old salt."),
        Some(
            "Warring: Failed to read iteration count from disk, will use the default value as old salt.",
        ),
    );

    let old_password_blob = match old_password {
        Some(p) => Some(create_password_blob(
            status.current_cipher,
            &old_salt,
            old_iteration_count,
            &p,
        )?),
        None => None,
    };

    device.change_encryption_passphrase(
        status.current_cipher,
        new_password_blob,
        old_password_blob,
    )?;

    write_security_block(
        device,
        new_iteration_count,
        new_salt,
        hint.unwrap_or_default(),
    )?;

    Ok(())
}

pub fn unwrap_salt_and_iteration_count(
    device: Option<&Scsi>,
    salt: Option<[u8; SALT_SIZE_FOR_U8]>,
    iteration_count: Option<u32>,
    default_salt_warning: Option<&str>,
    default_iteration_count_warning: Option<&str>,
) -> ([u8; SALT_SIZE_FOR_U8], u32) {
    let (block, from_disk) = match device {
        Some(device) => read_security_block_or_default(device),
        None => (
            SecurityBlock {
                iteration_count: DEFAULT_ITERATION_COUNT,
                salt: DEFAULT_SALT,
                hint: "".to_owned(),
            },
            false,
        ),
    };

    let salt = salt.unwrap_or_else(|| {
        if !from_disk {
            if let Some(warning) = default_salt_warning {
                eprintln!("{}", warning);
            }
        }

        block.salt
    });

    let iteration_count = iteration_count.unwrap_or_else(|| {
        if !from_disk {
            if let Some(warning) = default_iteration_count_warning {
                eprintln!("{}", warning);
            }
        }

        block.iteration_count
    });

    (salt, iteration_count)
}
