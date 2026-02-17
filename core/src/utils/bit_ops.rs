use crate::error::{BiometricError, Result};

pub fn xor_bits(a: &[u8], b: &[u8]) -> Result<Vec<u8>> {
    if a.len() != b.len() {
        return Err(BiometricError::Validation(format!(
            "xor requires same lengths, got {} and {}",
            a.len(),
            b.len()
        )));
    }
    Ok(a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect())
}

// Big-endian bit packing used for deterministic hash material handling.
pub fn pack_bits(bits: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(bits.len().div_ceil(8));
    let mut byte = 0u8;

    for (i, &bit) in bits.iter().enumerate() {
        if bit > 1 {
            return Err(BiometricError::InvalidBitValue(bit));
        }
        byte = (byte << 1) | bit;
        if i % 8 == 7 {
            out.push(byte);
            byte = 0;
        }
    }

    let rem = bits.len() % 8;
    if rem != 0 {
        byte <<= 8 - rem;
        out.push(byte);
    }

    Ok(out)
}

// Little-endian bit packing used for BCH backend compatibility.
pub fn pack_bits_le(bits: &[u8]) -> Result<Vec<u8>> {
    let mut out = vec![0u8; bits.len().div_ceil(8)];
    for (i, &bit) in bits.iter().enumerate() {
        if bit > 1 {
            return Err(BiometricError::InvalidBitValue(bit));
        }
        let byte_index = i / 8;
        let bit_index = i % 8;
        out[byte_index] |= bit << bit_index;
    }
    Ok(out)
}

pub fn unpack_bits(bytes: &[u8], bit_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(bit_len);
    for i in 0..bit_len {
        let byte = bytes[i / 8];
        let shift = 7 - (i % 8);
        out.push((byte >> shift) & 1);
    }
    out
}

pub fn unpack_bits_le(bytes: &[u8], bit_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(bit_len);
    for i in 0..bit_len {
        let byte = bytes[i / 8];
        let bit_index = i % 8;
        out.push((byte >> bit_index) & 1);
    }
    out
}
