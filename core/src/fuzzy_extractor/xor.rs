use crate::error::Result;
use crate::utils::bit_ops::xor_bits;

pub fn xor_vec(a: &[u8], b: &[u8]) -> Result<Vec<u8>> {
    xor_bits(a, b)
}
