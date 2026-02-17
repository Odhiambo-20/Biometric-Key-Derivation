use super::types::FfiResultCode;

#[no_mangle]
pub extern "C" fn bkd_android_init() -> FfiResultCode {
    FfiResultCode::OK
}
