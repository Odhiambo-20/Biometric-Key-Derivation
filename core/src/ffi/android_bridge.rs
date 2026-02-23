use core::ffi::c_void;

use super::types::{
    BkdEnrollRequest, BkdEnrollResponse, BkdRecoverRequest, BkdRecoverResponse, FfiResultCode,
};

#[no_mangle]
pub extern "C" fn bkd_android_init() -> FfiResultCode {
    FfiResultCode::OK
}

#[no_mangle]
/// # Safety
///
/// `req` and `resp` must satisfy the safety contract of `super::bkd_enroll`.
pub unsafe extern "C" fn bkd_android_enroll(
    req: *const BkdEnrollRequest,
    resp: *mut BkdEnrollResponse,
) -> FfiResultCode {
    unsafe { super::bkd_enroll(req, resp) }
}

#[no_mangle]
/// # Safety
///
/// `req` and `resp` must satisfy the safety contract of `super::bkd_recover`.
pub unsafe extern "C" fn bkd_android_recover(
    req: *const BkdRecoverRequest,
    resp: *mut BkdRecoverResponse,
) -> FfiResultCode {
    unsafe { super::bkd_recover(req, resp) }
}

#[allow(non_snake_case)]
#[no_mangle]
/// # Safety
///
/// JNI caller must pass valid pointers for request/response.
pub unsafe extern "C" fn Java_com_biometrickey_bridge_RustJNIBridge_enrollment(
    _env: *mut c_void,
    _class: *mut c_void,
    req: *const BkdEnrollRequest,
    resp: *mut BkdEnrollResponse,
) -> i32 {
    unsafe { super::bkd_enroll(req, resp).code }
}

#[allow(non_snake_case)]
#[no_mangle]
/// # Safety
///
/// JNI caller must pass valid pointers for request/response.
pub unsafe extern "C" fn Java_com_biometrickey_bridge_RustJNIBridge_recovery(
    _env: *mut c_void,
    _class: *mut c_void,
    req: *const BkdRecoverRequest,
    resp: *mut BkdRecoverResponse,
) -> i32 {
    unsafe { super::bkd_recover(req, resp).code }
}
