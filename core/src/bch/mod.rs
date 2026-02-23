mod backend;
pub mod decode;
pub mod encode;
pub mod params;

use backend::BchEngine;

use crate::error::{BiometricError, Result};
use crate::utils::bit_ops::{pack_bits_le, unpack_bits_le};
use crate::utils::validation::validate_bits;

pub use params::BchParams;

/// Galois field order for BCH
/// m=10 => GF(2^10) => supports codewords up to 1023 bits
const BCH_M: i32 = 10;

#[derive(Debug, Clone)]
pub struct BchCodec {
    pub params: BchParams,
}

impl BchCodec {
    pub fn new(params: BchParams) -> Self {
        Self { params }
    }

    /// Encode data bits into BCH codeword
    /// 
    /// Input: k=512 data bits
    /// Output: n=1023 codeword bits (512 data + 511 parity)
    pub fn encode(&self, data_bits: &[u8]) -> Result<Vec<u8>> {
        self.params.validate()?;

        if data_bits.len() != self.params.k {
            return Err(BiometricError::InvalidBitLength {
                expected: self.params.k,
                actual: data_bits.len(),
            });
        }

        validate_bits(data_bits, self.params.k)?;

        // Initialize BCH engine
        let engine = BchEngine::new(BCH_M, self.params.t as i32)?;
        
        // Pack bits into bytes for backend
        let data_bytes = pack_bits_le(data_bits)?;
        
        // Generate ECC parity bytes
        let ecc_bytes = engine.encode(&data_bytes)?;
        
        // Unpack ECC bytes back to bits
        let ecc_bits = unpack_bits_le(&ecc_bytes, ecc_bytes.len() * 8);

        // Construct codeword: [data bits | ecc bits | padding]
        let mut codeword = Vec::with_capacity(self.params.n);
        codeword.extend_from_slice(data_bits);
        codeword.extend_from_slice(&ecc_bits);

        // Pad to exactly n bits
        while codeword.len() < self.params.n {
            codeword.push(0);
        }

        // Truncate if needed (shouldn't happen with correct parameters)
        codeword.truncate(self.params.n);

        Ok(codeword)
    }

    /// Decode noisy BCH codeword with error correction
    /// 
    /// Input: n=1023 noisy codeword bits
    /// Output: k=512 corrected data bits
    pub fn decode(&self, noisy_codeword: &[u8]) -> Result<Vec<u8>> {
        self.params.validate()?;

        if noisy_codeword.len() != self.params.n {
            return Err(BiometricError::InvalidBitLength {
                expected: self.params.n,
                actual: noisy_codeword.len(),
            });
        }

        validate_bits(noisy_codeword, self.params.n)?;

        // Extract message and ECC portions
        let msg_bits = noisy_codeword[..self.params.k].to_vec();
        let ecc_len_bits = BCH_M as usize * self.params.t;
        let ecc_end = self.params.k + ecc_len_bits;
        let ecc_bits = noisy_codeword[self.params.k..ecc_end.min(self.params.n)].to_vec();

        // Pack into bytes for backend
        let mut msg_bytes = pack_bits_le(&msg_bits)?;
        let ecc_bytes = pack_bits_le(&ecc_bits)?;

        // Initialize BCH engine and perform error correction
        let engine = BchEngine::new(BCH_M, self.params.t as i32)?;
        let corrected_count = engine.decode_and_correct(&mut msg_bytes, &ecc_bytes)?;

        // Sanity check: corrected errors should not exceed t
        if corrected_count > self.params.t {
            return Err(BiometricError::EccDecode(format!(
                "Backend corrected {} errors, exceeding configured t={}. This should not happen.",
                corrected_count, self.params.t
            )));
        }

        // Unpack corrected bytes back to bits
        let recovered_bits = unpack_bits_le(&msg_bytes, self.params.k);
        validate_bits(&recovered_bits, self.params.k)?;

        Ok(recovered_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bch_encode_decode_no_errors() {
        let params = BchParams::new_1023_512(180);
        let codec = BchCodec::new(params);

        let data: Vec<u8> = (0..512).map(|i| (i % 2) as u8).collect();
        let codeword = codec.encode(&data).unwrap();
        assert_eq!(codeword.len(), 1023);

        let decoded = codec.decode(&codeword).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn bch_corrects_small_errors() {
        let params = BchParams::new_1023_512(180);
        let codec = BchCodec::new(params);

        let data: Vec<u8> = (0..512).map(|i| (i % 2) as u8).collect();
        let mut codeword = codec.encode(&data).unwrap();

        // Introduce 50 bit errors (well within t=180 tolerance)
        for i in 0..50 {
            codeword[i] ^= 1;
        }

        let decoded = codec.decode(&codeword).unwrap();
        assert_eq!(decoded, data);
    }
}
