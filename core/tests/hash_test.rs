use biometric_core::hash::hkdf::derive_key_256;
use biometric_core::hash::sha256::sha256_bytes;

#[test]
fn sha256_is_stable() {
    let digest = sha256_bytes(b"abc");
    assert_eq!(digest.len(), 32);
    assert_eq!(digest[0], 0xBA);
}

#[test]
fn hkdf_key_len_is_32() {
    let k = derive_key_256(b"ikm", b"salt-16-bytes!!!", b"info").unwrap();
    assert_eq!(k.len(), 32);
}
