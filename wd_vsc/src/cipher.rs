use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Cipher {
    NoEncryption,
    Aes128Ecb,
    Aes128Cbc,
    Aes128Xts,
    Aes256Ecb,
    Aes256Cbc,
    Aes256Xts,
    FullDiscEncryption,
    Unknown,
}

impl From<u8> for Cipher {
    fn from(code: u8) -> Self {
        match code {
            0x00 => Self::NoEncryption,
            0x10 => Self::Aes128Ecb,
            0x12 => Self::Aes128Cbc,
            0x18 => Self::Aes128Xts,
            0x20 => Self::Aes256Ecb,
            0x22 => Self::Aes256Cbc,
            0x28 => Self::Aes256Xts,
            0x30 => Self::FullDiscEncryption,
            _ => Self::Unknown,
        }
    }
}

impl From<Cipher> for u8 {
    fn from(cipher: Cipher) -> Self {
        match cipher {
            Cipher::NoEncryption => 0x00,
            Cipher::Aes128Ecb => 0x10,
            Cipher::Aes128Cbc => 0x12,
            Cipher::Aes128Xts => 0x18,
            Cipher::Aes256Ecb => 0x20,
            Cipher::Aes256Cbc => 0x22,
            Cipher::Aes256Xts => 0x28,
            Cipher::FullDiscEncryption => 0x30,
            Cipher::Unknown => 0xff,
        }
    }
}

impl Cipher {
    pub fn get_password_blob_size(&self) -> crate::Result<usize> {
        let password_size = match self {
            Cipher::NoEncryption => return Err(crate::Error::UnsupportedCipher),
            Cipher::Aes128Ecb => 16,
            Cipher::Aes128Cbc => 16,
            Cipher::Aes128Xts => 16,
            Cipher::Aes256Ecb => 32,
            Cipher::Aes256Cbc => 32,
            Cipher::Aes256Xts => 32,
            Cipher::FullDiscEncryption => 32,
            Cipher::Unknown => return Err(crate::Error::UnsupportedCipher),
        };

        Ok(password_size)
    }
}
