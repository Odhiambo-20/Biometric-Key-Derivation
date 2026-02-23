mod backend;
pub mod decode;
pub mod encode;
pub mod params;

use backend::BchEngine;

use crate::error::{BiometricError, Result};
use crate::utils::bit_ops::{pack_bits_le, unpack_bits_le};
use crate::utils::validation::validate_bits;

pub use params::BchParams;

const BCH_M: i32 = 10;

#[derive(Debug, Clone)]
pub struct BchCodec {
    pub params: BchParams,
}

impl BchCodec {
    pub fn new(params: BchParams) -> Self {
        Self { params }
    }

    pub fn encode(&self, data_bits: &[u8]) -> Result<Vec<u8>> {
        self.params.validate()?;

        if data_bits.len() != self.params.k {
            return Err(BiometricError::InvalidBitLength {
                expected: self.params.k,
                actual: data_bits.len(),
            });
        }

        validate_bits(data_bits, self.params.k)?;

        let engine = BchEngine::new(BCH_M, self.params.t as i32)?;
        let data_bytes = pack_bits_le(data_bits)?;
        let ecc_bytes = engine.encode(&data_bytes)?;
        let ecc_bits = unpack_bits_le(&ecc_bytes, ecc_bytes.len() * 8);

        let mut codeword = Vec::with_capacity(self.params.n);
        codeword.extend_from_slice(data_bits);
        codeword.extend_from_slice(&ecc_bits);

        // Fill remaining parity tail deterministically to keep fixed n=255 framing.
        while codeword.len() < self.params.n {
            codeword.push(0);
        }

        Ok(codeword)
    }

    pub fn decode(&self, noisy_codeword: &[u8]) -> Result<Vec<u8>> {
        self.params.validate()?;

        if noisy_codeword.len() != self.params.n {
            return Err(BiometricError::InvalidBitLength {
                expected: self.params.n,
                actual: noisy_codeword.len(),
            });
        }

        validate_bits(noisy_codeword, self.params.n)?;

        let msg_bits = noisy_codeword[..self.params.k].to_vec();
        let ecc_len_bits = BCH_M as usize * self.params.t;
        let ecc_end = self.params.k + ecc_len_bits;
        let ecc_bits = noisy_codeword[self.params.k..ecc_end].to_vec();

        let mut msg_bytes = pack_bits_le(&msg_bits)?;
        let ecc_bytes = pack_bits_le(&ecc_bits)?;

        let engine = BchEngine::new(BCH_M, self.params.t as i32)?;
        let corrected = engine.decode_and_correct(&mut msg_bytes, &ecc_bytes)?;
        if corrected > self.params.t {
            return Err(BiometricError::EccDecode(format!(
                "backend reported {} corrected bits above configured t={}",
                corrected, self.params.t
            )));
        }

        let recovered_bits = unpack_bits_le(&msg_bytes, self.params.k);
        validate_bits(&recovered_bits, self.params.k)?;

        Ok(recovered_bits)
    }
}
