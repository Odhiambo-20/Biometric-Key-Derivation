use crate::error::Result;

pub fn quantize_sign(embedding: &[f32]) -> Result<Vec<u8>> {
    Ok(embedding
        .iter()
        .map(|&v| if v >= 0.0 { 1u8 } else { 0u8 })
        .collect())
}
