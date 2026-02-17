use crate::error::{BiometricError, Result};

pub const EMBEDDING_DIM: usize = 128;

pub fn validate_embedding(embedding: &[f32]) -> Result<()> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(BiometricError::InvalidEmbeddingLength {
            expected: EMBEDDING_DIM,
            actual: embedding.len(),
        });
    }
    if embedding.iter().any(|v| !v.is_finite()) {
        return Err(BiometricError::Validation(
            "embedding contains non-finite float values".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_bits(bits: &[u8], expected: usize) -> Result<()> {
    if bits.len() != expected {
        return Err(BiometricError::InvalidBitLength {
            expected,
            actual: bits.len(),
        });
    }
    for &bit in bits {
        if bit > 1 {
            return Err(BiometricError::InvalidBitValue(bit));
        }
    }
    Ok(())
}
