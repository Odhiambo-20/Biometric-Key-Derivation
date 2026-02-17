use biometric_core::{enroll, recover, BchParams, QuantizationMethod};

fn base_embedding() -> Vec<f32> {
    // Deterministic 128-d vector.
    (0..128)
        .map(|i| if i % 3 == 0 { 0.6 } else { -0.4 })
        .collect()
}

#[test]
fn enrollment_and_recovery_same_embedding() {
    let emb = base_embedding();
    let params = BchParams::new_255_128(90);

    let out = enroll(&emb, QuantizationMethod::Sign, params).unwrap();
    let recovered = recover(&emb, QuantizationMethod::Sign, &out.helper_data).unwrap();

    assert_eq!(out.crypto_key, recovered);
}

#[test]
fn recovery_fails_for_large_variation() {
    let emb = base_embedding();
    let params = BchParams::new_255_128(20);

    let out = enroll(&emb, QuantizationMethod::Sign, params).unwrap();

    let mut changed = emb.clone();
    for item in changed.iter_mut().take(80) {
        *item *= -1.0;
    }

    let recovered = recover(&changed, QuantizationMethod::Sign, &out.helper_data);
    assert!(recovered.is_err());
}
