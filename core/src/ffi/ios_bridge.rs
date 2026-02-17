use super::types::{BkdEnrollRequest, BkdEnrollResponse, BkdRecoverRequest, BkdRecoverResponse, FfiResultCode};

#[no_mangle]
pub extern "C" fn bkd_ios_init() -> FfiResultCode {
    FfiResultCode::OK
}

#[no_mangle]
pub extern "C" fn bkd_ios_enroll(
    req: *const BkdEnrollRequest,
    resp: *mut BkdEnrollResponse,
) -> FfiResultCode {
    super::bkd_enroll(req, resp)
}

#[no_mangle]
pub extern "C" fn bkd_ios_recover(
    req: *const BkdRecoverRequest,
    resp: *mut BkdRecoverResponse,
) -> FfiResultCode {
    super::bkd_recover(req, resp)
}
