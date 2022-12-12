use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Device is not locked. {0}")]
    NotLocked(String),
    #[error("Device is not unlocked. {0}")]
    NotUnlocked(String),
    #[error("Exceeded the maxium unlock attempts.")]
    ExceedUnlockAttempts,
    #[error("Password incorrect.")]
    PasswordIncorrect,
    #[error("Password blob size incorrect.")]
    PasswordBlobSizeIncorrect,
    #[error("Unsupported cipher.")]
    UnsupportedCipher,
    #[error("LibScsi Error. {0:?}")]
    ScsiError(#[from] libscsi::Error),
    #[error("IO Error. {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}
