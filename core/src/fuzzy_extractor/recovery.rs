use subtle::ConstantTimeEq;
use zeroize::Zeroize;

use crate::bch::{encode::expand_biometric_bits, BchCodec, BchParams};
use crate::error::{BiometricError, Result};
use crate::fuzzy_extractor::helper_data::HelperData;
use crate::fuzzy_extractor::xor::xor_vec;
use crate::hash::hkdf::derive_key_256;
use crate::hash::sha256::sha256_bytes;
use crate::quantization::{quantize_embedding, QuantizationMethod};
use crate::utils::bit_ops::pack_bits;
use crate::utils::validation::validate_bits;

use super::enrollment::HKDF_INFO;

pub fn recover(
    embedding: &[f32],
    method: QuantizationMethod,
    helper: &HelperData,
) -> Result<[u8; 32]> {
    let params = BchParams {
        n: helper.n,
        k: helper.k,
        t: helper.t,
    };
    params.validate()?;

    let codec = BchCodec::new(params);

    validate_bits(&helper.helper_bits, helper.n)?;

    let biometric_bits = quantize_embedding(embedding, method)?;
    let expanded_bio = expand_biometric_bits(&biometric_bits, helper.n)?;
    let noisy_codeword = xor_vec(&helper.helper_bits, &expanded_bio)?;

    let mut recovered_message = codec.decode(&noisy_codeword)?;
    let mut recovered_bytes = pack_bits(&recovered_message)?;
    let commitment = sha256_bytes(&recovered_bytes);

    if commitment.ct_eq(&helper.commitment).unwrap_u8() != 1 {
        recovered_message.zeroize();
        recovered_bytes.zeroize();
        return Err(BiometricError::CommitmentMismatch);
    }

    let key = derive_key_256(&recovered_bytes, &helper.salt, HKDF_INFO)?;

    recovered_message.zeroize();
    recovered_bytes.zeroize();

    Ok(key)
}
