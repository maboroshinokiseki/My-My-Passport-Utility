use std::{any::Any, io, path::PathBuf};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error<T = Box<dyn Any>> {
    #[error("{0} is not a block device.")]
    NotBlockDevice(PathBuf),
    #[error("{0} is not an SCSI Generic device, or old SCSI Generic driver.")]
    NotScsiDevice(PathBuf),
    #[error("Check condition: {0:?}")]
    CheckCondition(T),
    #[error("{0:?}")]
    Other(String),
    #[error("{0}")]
    IO(#[from] io::Error),
}
