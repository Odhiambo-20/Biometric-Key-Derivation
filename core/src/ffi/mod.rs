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
const HELPER_HDR_LEN: usize = 1 + 4 + 4 + 4 + 16 + 32 + 4;

fn method_from_ffi(method: u32, threshold: f32) -> Option<QuantizationMethod> {
    match method {
        0 => Some(QuantizationMethod::Sign),
        1 => Some(QuantizationMethod::Threshold(threshold)),
        2 => Some(QuantizationMethod::MultiBit2),
        _ => None,
    }
}

fn serialize_helper_data(helper: &HelperData) -> Vec<u8> {
    let helper_len = helper.helper_bits.len() as u32;
    let mut out = Vec::with_capacity(HELPER_HDR_LEN + helper.helper_bits.len());

    out.push(helper.version);
    out.extend_from_slice(&(helper.n as u32).to_le_bytes());
    out.extend_from_slice(&(helper.k as u32).to_le_bytes());
    out.extend_from_slice(&(helper.t as u32).to_le_bytes());
    out.extend_from_slice(&helper.salt);
    out.extend_from_slice(&helper.commitment);
    out.extend_from_slice(&helper_len.to_le_bytes());
    out.extend_from_slice(&helper.helper_bits);

    out
}

fn deserialize_helper_data(data: &[u8]) -> Option<HelperData> {
    if data.len() < HELPER_HDR_LEN {
        return None;
    }

    let version = data[0];
    let n = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
    let k = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
    let t = u32::from_le_bytes([data[9], data[10], data[11], data[12]]) as usize;

    let mut salt = [0u8; 16];
    salt.copy_from_slice(&data[13..29]);

    let mut commitment = [0u8; 32];
    commitment.copy_from_slice(&data[29..61]);

    let helper_len = u32::from_le_bytes([data[61], data[62], data[63], data[64]]) as usize;
    if data.len() != HELPER_HDR_LEN + helper_len {
        return None;
    }

    let helper_bits = data[65..].to_vec();

    Some(HelperData {
        version,
        n,
        k,
        t,
        helper_bits,
        commitment,
        salt,
    })
}

#[no_mangle]
pub extern "C" fn bkd_enroll(
    req: *const BkdEnrollRequest,
    resp: *mut BkdEnrollResponse,
) -> FfiResultCode {
    if req.is_null() || resp.is_null() {
        return FfiResultCode::ERR_NULL;
    }

    let req = unsafe { &*req };
    let resp = unsafe { &mut *resp };

    if req.embedding_ptr.is_null()
        || resp.helper_out_ptr.is_null()
        || resp.helper_out_len_ptr.is_null()
        || resp.key_out_ptr.is_null()
    {
        return FfiResultCode::ERR_NULL;
    }

    if resp.key_out_cap < KEY_LEN {
        return FfiResultCode::ERR_BUFFER_TOO_SMALL;
    }

    let method = match method_from_ffi(req.method, req.threshold) {
        Some(m) => m,
        None => return FfiResultCode::ERR_INVALID,
    };

    let embedding = unsafe { slice::from_raw_parts(req.embedding_ptr, req.embedding_len) };
    let params = BchParams::new_255_128(req.bch_t);
    if params.validate().is_err() {
        return FfiResultCode::ERR_INVALID;
    }

    let out = match enroll(embedding, method, params) {
        Ok(v) => v,
        Err(_) => return FfiResultCode::ERR_INTERNAL,
    };

    let helper_blob = serialize_helper_data(&out.helper_data);
    if resp.helper_out_cap < helper_blob.len() {
        return FfiResultCode::ERR_BUFFER_TOO_SMALL;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(helper_blob.as_ptr(), resp.helper_out_ptr, helper_blob.len());
        *resp.helper_out_len_ptr = helper_blob.len();
        std::ptr::copy_nonoverlapping(out.crypto_key.as_ptr(), resp.key_out_ptr, KEY_LEN);
    }

    FfiResultCode::OK
}

#[no_mangle]
pub extern "C" fn bkd_recover(
    req: *const BkdRecoverRequest,
    resp: *mut BkdRecoverResponse,
) -> FfiResultCode {
    if req.is_null() || resp.is_null() {
        return FfiResultCode::ERR_NULL;
    }

    let req = unsafe { &*req };
    let resp = unsafe { &mut *resp };

    if req.embedding_ptr.is_null() || req.helper_ptr.is_null() || resp.key_out_ptr.is_null() {
        return FfiResultCode::ERR_NULL;
    }

    if resp.key_out_cap < KEY_LEN {
        return FfiResultCode::ERR_BUFFER_TOO_SMALL;
    }

    let method = match method_from_ffi(req.method, req.threshold) {
        Some(m) => m,
        None => return FfiResultCode::ERR_INVALID,
    };

    let embedding = unsafe { slice::from_raw_parts(req.embedding_ptr, req.embedding_len) };
    let helper_blob = unsafe { slice::from_raw_parts(req.helper_ptr, req.helper_len) };

    let helper = match deserialize_helper_data(helper_blob) {
        Some(h) => h,
        None => return FfiResultCode::ERR_SERIALIZATION,
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
