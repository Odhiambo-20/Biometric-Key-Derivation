mod backend;
pub mod decode;
pub mod encode;
pub mod params;

use backend::BchEngine;

use crate::error::{BiometricError, Result};
use crate::utils::bit_ops::{pack_bits, unpack_bits};
use crate::utils::validation::validate_bits;

pub use params::BchParams;

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

        let data_bytes = pack_bits(data_bits)?;
        let engine = BchEngine::new(8, 16)?;
        let ecc = engine.encode(&data_bytes)?;
        let ecc_bits = unpack_bits(&ecc, ecc.len() * 8);

        // API compatibility: keep n=255. Linux BCH(8,16) with 16-byte input yields
        // 128 data bits + 128 parity bits. We drop one parity bit to keep 255 bits.
        let mut codeword = Vec::with_capacity(self.params.n);
        codeword.extend_from_slice(data_bits);
        codeword.extend_from_slice(&ecc_bits[..127]);

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

        let mut msg_bits = noisy_codeword[..128].to_vec();
        let mut ecc_bits = noisy_codeword[128..].to_vec();

        // Restore dropped parity bit as 0 for backend call.
        ecc_bits.push(0);

        let mut msg_bytes = pack_bits(&msg_bits)?;
        let ecc_bytes = pack_bits(&ecc_bits)?;

        let engine = BchEngine::new(8, 16)?;
        let _ = engine.decode_and_correct(&mut msg_bytes, &ecc_bytes)?;

        msg_bits = unpack_bits(&msg_bytes, 128);
        validate_bits(&msg_bits, 128)?;

        Ok(msg_bits)
    }
}
