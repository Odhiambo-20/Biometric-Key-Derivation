use biometric_core::hash::sha256::sha256_bytes;

#[test]
fn helper_commitment_hash_is_non_zero() {
    let d = sha256_bytes(b"biometric-commitment");
    assert!(d.iter().any(|b| *b != 0));
}
