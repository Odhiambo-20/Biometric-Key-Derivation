use biometric_core::{enroll, recover, BchParams, QuantizationMethod};

fn base_embedding() -> Vec<f32> {
    (0..512)
        .map(|i| if i % 3 == 0 { 0.6 } else { -0.4 })
        .collect()
}

#[test]
fn enrollment_and_recovery_same_embedding() {
    let emb = base_embedding();
    let params = BchParams::new_2047_512(180);

    let out = enroll(&emb, QuantizationMethod::Sign, params).unwrap();
    let recovered = recover(&emb, QuantizationMethod::Sign, &out.helper_data).unwrap();

    assert_eq!(out.crypto_key, recovered);
}

#[test]
fn recovery_fails_for_large_variation() {
    let emb = base_embedding();
    let params = BchParams::new_2047_512(180);

    let out = enroll(&emb, QuantizationMethod::Sign, params).unwrap();

    // Flip the sign of 220 out of 512 embedding values.
    // After sign-based quantization this produces 220 flipped bits out of 512,
    // which is 42.9% bit error rate. This far exceeds t=180 (~8.8% of 2047),
    // so BCH decode must fail and recovery must return an error.
    let mut changed = emb.clone();
    for item in changed.iter_mut().take(220) {
        *item *= -1.0;
    }

    let recovered = recover(&changed, QuantizationMethod::Sign, &out.helper_data);
    assert!(recovered.is_err());
}
