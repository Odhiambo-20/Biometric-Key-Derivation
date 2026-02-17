use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroize;

use crate::bch::{encode::expand_biometric_bits, BchCodec, BchParams};
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

pub fn enroll(
    embedding: &[f32],
    method: QuantizationMethod,
    params: BchParams,
) -> Result<EnrollmentOutput> {
    params.validate()?;
    let codec = BchCodec::new(params);

    let biometric_bits = quantize_embedding(embedding, method)?;
    let expanded_bio = expand_biometric_bits(&biometric_bits, params.n)?;

    let mut message_bits = vec![0u8; params.k];
    let mut rnd = vec![0u8; params.k];
    OsRng.fill_bytes(&mut rnd);
    for (dst, byte) in message_bits.iter_mut().zip(rnd.iter()) {
        *dst = byte & 1;
    }

    let codeword = codec.encode(&message_bits)?;
    let helper_bits = xor_vec(&codeword, &expanded_bio)?;

    let mut message_bytes = pack_bits(&message_bits)?;
    let commitment = sha256_bytes(&message_bytes);

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let crypto_key = derive_key_256(&message_bytes, &salt, HKDF_INFO)?;

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
