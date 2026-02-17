use super::types::FfiResultCode;

#[no_mangle]
pub extern "C" fn bkd_ios_init() -> FfiResultCode {
    FfiResultCode::OK
}
