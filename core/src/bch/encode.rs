use crate::error::{BiometricError, Result};
use crate::utils::validation::validate_bits;

pub fn expand_biometric_bits(bits: &[u8], target_len: usize) -> Result<Vec<u8>> {
    if bits.is_empty() {
        return Err(BiometricError::Validation(
            "biometric bit vector cannot be empty".to_string(),
        ));
    }
    validate_bits(bits, bits.len())?;

    if target_len < bits.len() {
        return Err(BiometricError::Validation(format!(
            "target_len {} must be >= source_len {}",
            target_len,
            bits.len()
        )));
    }

    let mut out = Vec::with_capacity(target_len);
    out.extend_from_slice(bits);

    let mut i = 0usize;
    while out.len() < target_len {
        out.push(bits[i % bits.len()]);
        i += 1;
    }

    Ok(out)
}

// Kept for API compatibility.
pub fn expand_biometric_bits_255(bits_128: &[u8]) -> Result<Vec<u8>> {
    expand_biometric_bits(bits_128, 256)
}
