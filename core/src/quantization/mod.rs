pub mod multibit;
pub mod sign_based;
pub mod threshold;

use crate::error::Result;
use crate::utils::validation::validate_embedding;

#[derive(Debug, Clone, Copy)]
pub enum QuantizationMethod {
    Sign,
    Threshold(f32),
    MultiBit2,
}

pub fn quantize_embedding(embedding: &[f32], method: QuantizationMethod) -> Result<Vec<u8>> {
    validate_embedding(embedding)?;
    match method {
        QuantizationMethod::Sign => sign_based::quantize_sign(embedding),
        QuantizationMethod::Threshold(t) => threshold::quantize_threshold(embedding, t),
        QuantizationMethod::MultiBit2 => multibit::quantize_2bit(embedding),
    }
}
