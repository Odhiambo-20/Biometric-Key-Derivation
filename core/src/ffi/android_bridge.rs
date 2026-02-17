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
