use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroize;

use crate::bch::{BchCodec, BchParams};
use crate::error::Result;
use crate::fuzzy_extractor::helper_data::HelperData;
use crate::fuzzy_extractor::xor::xor_vec;
use crate::hash::hkdf::derive_key_256;
use crate::hash::sha256::sha256_bytes;
use crate::quantization::{quantize_embedding, QuantizationMethod};
use crate::utils::bit_ops::pack_bits;

pub const HKDF_INFO: &[u8] = b"biometric-core/v1/key-derivation";

#[derive(Debug, Clone)]
pub struct EnrollmentOutput {
    pub helper_data: HelperData,
    pub crypto_key: [u8; 32],
}

/// Enrollment: Convert face embedding into helper data and cryptographic key
/// 
/// Steps:
/// 1. Quantize 512-float embedding → 512 bits
/// 2. Generate random 512-bit message
/// 3. BCH encode message → 1023-bit codeword
/// 4. XOR codeword with quantized biometric → helper data (store publicly)
/// 5. Hash message → commitment (for verification)
/// 6. HKDF on message → 256-bit crypto key
/// 7. Securely delete message and intermediate values
pub fn enroll(
    embedding: &[f32],
    method: QuantizationMethod,
    params: BchParams,
) -> Result<EnrollmentOutput> {
    params.validate()?;
    let codec = BchCodec::new(params);

    // Step 1: Quantize embedding to bits
    let biometric_bits = quantize_embedding(embedding, method)?;
    
    if biometric_bits.len() != params.k {
        // For 512-dim embedding with sign quantization, should be 512 bits
        // But if using multi-bit quantization, might be different
        // Pad or truncate to match k
        let mut adjusted_bio = biometric_bits.clone();
        adjusted_bio.resize(params.k, 0);
        return Self::enroll_internal(adjusted_bio, params, codec);
    }

    Self::enroll_internal(biometric_bits, params, codec)
}

impl EnrollmentOutput {
    fn enroll_internal(
        biometric_bits: Vec<u8>,
        params: BchParams,
        codec: BchCodec,
    ) -> Result<EnrollmentOutput> {
        // Step 2: Generate random message (k bits)
        let mut message_bits = vec![0u8; params.k];
        let mut rnd = vec![0u8; (params.k + 7) / 8];
        OsRng.fill_bytes(&mut rnd);
        for (i, dst) in message_bits.iter_mut().enumerate() {
            *dst = (rnd[i / 8] >> (i % 8)) & 1;
        }

        // Step 3: BCH encode message → codeword (n bits)
        let codeword = codec.encode(&message_bits)?;

        // Expand biometric bits to match codeword length if needed
        let mut bio_expanded = biometric_bits.clone();
        while bio_expanded.len() < params.n {
            // Simple repetition padding
            let idx = bio_expanded.len() % biometric_bits.len();
            bio_expanded.push(biometric_bits[idx]);
        }
        bio_expanded.truncate(params.n);

        // Step 4: XOR codeword with biometric → helper data
        let helper_bits = xor_vec(&codeword, &bio_expanded)?;

        // Step 5: Commitment (hash of message for verification)
        let mut message_bytes = pack_bits(&message_bits)?;
        let commitment = sha256_bytes(&message_bytes);

        // Step 6: Generate salt and derive key
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let crypto_key = derive_key_256(&message_bytes, &salt, HKDF_INFO)?;

        // Step 7: Secure deletion
        rnd.zeroize();
        message_bits.zeroize();
        message_bytes.zeroize();

        let helper_data = HelperData {
            version: 1,
            n: params.n,
            k: params.k,
            t: params.t,
            helper_bits,
            commitment,
            salt,
        };

        Ok(EnrollmentOutput {
            helper_data,
            crypto_key,
        })
    }
}
