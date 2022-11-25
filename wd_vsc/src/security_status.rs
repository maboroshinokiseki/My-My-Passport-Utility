use std::io;

#[derive(Debug)]
pub enum SecurityStatus {
    /// it means it's been encrypted with a default password
    NoUserPassword,
    Locked,
    Unlocked,
    UnlockAttemptExceeded,
    NoEncryption,
}

impl TryFrom<u8> for SecurityStatus {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let security_status = match value {
            0 => Self::NoUserPassword,
            1 => Self::Locked,
            2 => Self::Unlocked,
            6 => Self::UnlockAttemptExceeded,
            7 => Self::NoEncryption,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unknown security status, unsupported device.",
                ));
            }
        };

        Ok(security_status)
    }
}
