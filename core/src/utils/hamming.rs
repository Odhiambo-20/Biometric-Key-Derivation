use crate::error::{BiometricError, Result};

pub fn hamming_distance(a: &[u8], b: &[u8]) -> Result<usize> {
    if a.len() != b.len() {
        return Err(BiometricError::Validation(format!(
            "hamming distance requires same lengths, got {} and {}",
            a.len(),
            b.len()
        )));
    }
    Ok(a.iter().zip(b.iter()).filter(|(x, y)| x != y).count())
}
