use crate::error::Result;
use crate::utils::validation::validate_bits;

pub fn encode_repetition_255_from_128(data_bits: &[u8]) -> Result<Vec<u8>> {
    validate_bits(data_bits, 128)?;
    Ok(data_bits.to_vec())
}

pub fn expand_biometric_bits_255(bits_128: &[u8]) -> Result<Vec<u8>> {
    validate_bits(bits_128, 128)?;

    // For helper-data XOR alignment we need 255 bits. We deterministically expand
    // 128 biometric bits by appending 127 mirrored bits.
    let mut out = Vec::with_capacity(255);
    out.extend_from_slice(bits_128);
    out.extend(bits_128.iter().copied().take(127));
    Ok(out)
}
