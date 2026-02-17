mod backend;
pub mod decode;
pub mod encode;
pub mod params;

use backend::BchEngine;

use crate::error::{BiometricError, Result};
use crate::utils::bit_ops::{pack_bits_le, unpack_bits_le};
use crate::utils::hamming::hamming_distance;
use crate::utils::validation::validate_bits;

pub use params::BchParams;

const BCH_M: i32 = 8;
const BCH_T_INTERNAL: i32 = 16;

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

        let engine = BchEngine::new(BCH_M, BCH_T_INTERNAL)?;
        let data_bytes = pack_bits_le(data_bits)?;
        let ecc_bytes = engine.encode(&data_bytes)?;
        let ecc_bits_full = unpack_bits_le(&ecc_bytes, ecc_bytes.len() * 8);

        // Produce fixed n=255 codeword: 128 data bits + first 127 parity bits.
        let mut codeword = Vec::with_capacity(self.params.n);
        codeword.extend_from_slice(data_bits);
        codeword.extend_from_slice(&ecc_bits_full[..127]);

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
        let mut ecc_bits = noisy_codeword[self.params.k..].to_vec();
        ecc_bits.push(0);

        let mut msg_bytes = pack_bits_le(&msg_bits)?;
        let ecc_bytes = pack_bits_le(&ecc_bits)?;

        let engine = BchEngine::new(BCH_M, BCH_T_INTERNAL)?;
        let _corrected = engine.decode_and_correct(&mut msg_bytes, &ecc_bytes)?;

        let recovered_bits = unpack_bits_le(&msg_bytes, self.params.k);
        validate_bits(&recovered_bits, self.params.k)?;

        // Enforce caller-defined tolerance window on re-encoded distance.
        let reencoded = self.encode(&recovered_bits)?;
        let distance = hamming_distance(noisy_codeword, &reencoded)?;
        if distance > self.params.t {
            return Err(BiometricError::EccDecode(format!(
                "distance {} exceeds configured tolerance t={}",
                distance, self.params.t
            )));
        }

        Ok(recovered_bits)
    }
}
