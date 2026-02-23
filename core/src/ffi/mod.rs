pub mod android_bridge;
pub mod ios_bridge;
pub mod types;

use std::slice;

use crate::bch::BchParams;
use crate::fuzzy_extractor::helper_data::HelperData;
use crate::quantization::QuantizationMethod;
use crate::{enroll, recover};

use self::types::{
    BkdEnrollRequest, BkdEnrollResponse, BkdRecoverRequest, BkdRecoverResponse, FfiResultCode,
};

const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const COMMITMENT_LEN: usize = 32;
/// Codeword length for BCH(2047, 512, 180) with m=11.
/// Changed from 1023 (m=10, which violated the BCH bound t*m <= n).
const HELPER_BITS_LEN: usize = 2047;

fn method_from_ffi(method: u32, threshold: f32) -> Option<QuantizationMethod> {
    match method {
        0 => Some(QuantizationMethod::Sign),
        1 => Some(QuantizationMethod::Threshold(threshold)),
        2 => Some(QuantizationMethod::MultiBit2),
        _ => None,
    }
}

#[no_mangle]
/// # Safety
///
/// `req` and `resp` must be valid non-null pointers to initialized
/// `BkdEnrollRequest`/`BkdEnrollResponse` structures for the duration
/// of this call. Embedded buffer pointers must be valid for the
/// specified capacities.
pub unsafe extern "C" fn bkd_enroll(
    req: *const BkdEnrollRequest,
    resp: *mut BkdEnrollResponse,
) -> FfiResultCode {
    if req.is_null() || resp.is_null() {
        return FfiResultCode::ERR_NULL;
    }

    let req = unsafe { &*req };
    let resp = unsafe { &mut *resp };

    if req.embedding_ptr.is_null()
        || resp.helper_bits_out_ptr.is_null()
        || resp.salt_out_ptr.is_null()
        || resp.commitment_out_ptr.is_null()
        || resp.key_out_ptr.is_null()
    {
        return FfiResultCode::ERR_NULL;
    }

    if resp.helper_bits_out_cap < HELPER_BITS_LEN
        || resp.salt_out_cap < SALT_LEN
        || resp.commitment_out_cap < COMMITMENT_LEN
        || resp.key_out_cap < KEY_LEN
    {
        return FfiResultCode::ERR_BUFFER_TOO_SMALL;
    }

    let method = match method_from_ffi(req.method, req.threshold) {
        Some(m) => m,
        None => return FfiResultCode::ERR_INVALID,
    };

    let embedding = unsafe { slice::from_raw_parts(req.embedding_ptr, req.embedding_len) };
    let params = BchParams::new_2047_512(req.bch_t);
    if params.validate().is_err() {
        return FfiResultCode::ERR_INVALID;
    }

    let out = match enroll(embedding, method, params) {
        Ok(v) => v,
        Err(_) => return FfiResultCode::ERR_INTERNAL,
    };

    if out.helper_data.helper_bits.len() != HELPER_BITS_LEN {
        return FfiResultCode::ERR_INTERNAL;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            out.helper_data.helper_bits.as_ptr(),
            resp.helper_bits_out_ptr,
            HELPER_BITS_LEN,
        );
        std::ptr::copy_nonoverlapping(out.helper_data.salt.as_ptr(), resp.salt_out_ptr, SALT_LEN);
        std::ptr::copy_nonoverlapping(
            out.helper_data.commitment.as_ptr(),
            resp.commitment_out_ptr,
            COMMITMENT_LEN,
        );
        std::ptr::copy_nonoverlapping(out.crypto_key.as_ptr(), resp.key_out_ptr, KEY_LEN);
    }

    FfiResultCode::OK
}

#[no_mangle]
/// # Safety
///
/// `req` and `resp` must be valid non-null pointers to initialized
/// `BkdRecoverRequest`/`BkdRecoverResponse` structures for the duration
/// of this call. Embedded buffer pointers must be valid for the
/// specified capacities.
pub unsafe extern "C" fn bkd_recover(
    req: *const BkdRecoverRequest,
    resp: *mut BkdRecoverResponse,
) -> FfiResultCode {
    if req.is_null() || resp.is_null() {
        return FfiResultCode::ERR_NULL;
    }

    let req = unsafe { &*req };
    let resp = unsafe { &mut *resp };

    if req.embedding_ptr.is_null()
        || req.helper_bits_ptr.is_null()
        || req.salt_ptr.is_null()
        || req.commitment_ptr.is_null()
        || resp.key_out_ptr.is_null()
    {
        return FfiResultCode::ERR_NULL;
    }

    if req.helper_bits_len != HELPER_BITS_LEN
        || req.salt_len != SALT_LEN
        || req.commitment_len != COMMITMENT_LEN
        || resp.key_out_cap < KEY_LEN
    {
        return FfiResultCode::ERR_INVALID;
    }

    let method = match method_from_ffi(req.method, req.threshold) {
        Some(m) => m,
        None => return FfiResultCode::ERR_INVALID,
    };

    let embedding = unsafe { slice::from_raw_parts(req.embedding_ptr, req.embedding_len) };
    let helper_bits = unsafe { slice::from_raw_parts(req.helper_bits_ptr, req.helper_bits_len) };
    let salt = unsafe { slice::from_raw_parts(req.salt_ptr, req.salt_len) };
    let commitment = unsafe { slice::from_raw_parts(req.commitment_ptr, req.commitment_len) };

    let mut salt_arr = [0u8; SALT_LEN];
    salt_arr.copy_from_slice(salt);
    let mut commitment_arr = [0u8; COMMITMENT_LEN];
    commitment_arr.copy_from_slice(commitment);

    let params = BchParams::new_2047_512(req.bch_t);
    if params.validate().is_err() {
        return FfiResultCode::ERR_INVALID;
    }

    let helper = HelperData {
        version: 1,
        n: params.n,
        k: params.k,
        t: params.t,
        helper_bits: helper_bits.to_vec(),
        commitment: commitment_arr,
        salt: salt_arr,
    };

    let key = match recover(embedding, method, &helper) {
        Ok(v) => v,
        Err(_) => return FfiResultCode::ERR_INTERNAL,
    };

    unsafe {
        std::ptr::copy_nonoverlapping(key.as_ptr(), resp.key_out_ptr, KEY_LEN);
    }

    FfiResultCode::OK
}
