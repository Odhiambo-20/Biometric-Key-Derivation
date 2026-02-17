use biometric_core::ffi::{android_bridge::bkd_android_init, ios_bridge::bkd_ios_init};

#[test]
fn ffi_entry_points_exist() {
    assert_eq!(bkd_ios_init().code, 0);
    assert_eq!(bkd_android_init().code, 0);
}
