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

/// Enrollment: Convert a mean face embedding into helper data and a cryptographic key.
///
/// The caller is responsible for passing a mean embedding computed from 10-15 AdaFace
/// inference results (element-wise average across frames from the same video).
/// This averaging step reduces same-person bit errors from 7-15% (single frame)
/// to 3-8% (mean of 10+ frames), which is within the BCH(2047, 512, 180) tolerance.
///
/// Pipeline:
///   1. Quantize 512-float mean embedding to 512 bits (sign-based: >= 0.0 => 1, else 0)
///   2. Generate a cryptographically random 512-bit message
///   3. BCH encode the message into a 2047-bit codeword
///   4. XOR the codeword with the (zero-padded) biometric bits => helper data (stored publicly)
///   5. Hash the message to produce a commitment (stored for verification)
///   6. Generate a random 16-byte salt, derive 256-bit key via HKDF(message, salt)
///   7. Securely zero all intermediate secrets
///
/// Stored (public, non-sensitive): helper_data (helper_bits, commitment, salt, n, k, t)
/// Derived (never stored):         crypto_key
pub fn enroll(
    embedding: &[f32],
    method: QuantizationMethod,
    params: BchParams,
) -> Result<EnrollmentOutput> {
    params.validate()?;
    let codec = BchCodec::new(params);

    // Step 1: Quantize the mean embedding to bits
    let biometric_bits = quantize_embedding(embedding, method)?;

    // Adjust length to match k if quantization produced a different count.
    // For 512-dim embeddings with sign-based quantization this should always be 512.
    let mut adjusted_bio = biometric_bits.clone();
    adjusted_bio.resize(params.k, 0);

    enroll_internal(adjusted_bio, params, codec)
}

fn enroll_internal(
    biometric_bits: Vec<u8>,
    params: BchParams,
    codec: BchCodec,
) -> Result<EnrollmentOutput> {
    // Step 2: Generate a random k-bit message
    let mut message_bits = vec![0u8; params.k];
    // Use div_ceil to compute the exact byte count needed for k bits
    let mut rnd = vec![0u8; params.k.div_ceil(8)];
    OsRng.fill_bytes(&mut rnd);
    for (i, dst) in message_bits.iter_mut().enumerate() {
        *dst = (rnd[i / 8] >> (i % 8)) & 1;
    }

    // Step 3: BCH encode the message into an n-bit codeword
    let codeword = codec.encode(&message_bits)?;

    // Step 4: Expand biometric bits to n bits (repetition padding) then XOR with codeword
    let mut bio_expanded = biometric_bits.clone();
    while bio_expanded.len() < params.n {
        let idx = bio_expanded.len() % biometric_bits.len();
        bio_expanded.push(biometric_bits[idx]);
    }
    bio_expanded.truncate(params.n);

    let helper_bits = xor_vec(&codeword, &bio_expanded)?;

    // Step 5: Commitment = SHA-256(message bytes)
    let mut message_bytes = pack_bits(&message_bits)?;
    let commitment = sha256_bytes(&message_bytes);

    // Step 6: Derive 256-bit crypto key via HKDF(message, salt)
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let crypto_key = derive_key_256(&message_bytes, &salt, HKDF_INFO)?;

    // Step 7: Securely zero all intermediate secret material
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
