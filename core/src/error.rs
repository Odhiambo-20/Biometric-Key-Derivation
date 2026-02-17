use thiserror::Error;

#[derive(Debug, Error)]
pub enum BiometricError {
    #[error("invalid embedding length: expected {expected}, got {actual}")]
    InvalidEmbeddingLength { expected: usize, actual: usize },

    #[error("invalid bit vector length: expected {expected}, got {actual}")]
    InvalidBitLength { expected: usize, actual: usize },

    #[error("invalid bit value: {0}. Bits must be 0 or 1")]
    InvalidBitValue(u8),

    #[error("quantization failed: {0}")]
    Quantization(String),

    #[error("invalid BCH params: {0}")]
    InvalidBchParams(String),

    #[error("ecc decode failed: {0}")]
    EccDecode(String),

    #[error("commitment mismatch: biometric sample is outside tolerated range")]
    CommitmentMismatch,

    #[error("insecure configuration: {0}")]
    InsecureConfiguration(String),

    #[error("input validation failed: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, BiometricError>;
