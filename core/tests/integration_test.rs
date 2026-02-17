use biometric_core::{enroll, recover, BchParams, QuantizationMethod};

#[test]
fn full_pipeline_returns_256_bit_key() {
    let emb: Vec<f32> = (0..128).map(|i| ((i as f32) / 128.0) - 0.5).collect();

    let out = enroll(&emb, QuantizationMethod::Sign, BchParams::default()).unwrap();
    let recovered = recover(&emb, QuantizationMethod::Sign, &out.helper_data).unwrap();

    assert_eq!(out.crypto_key.len(), 32);
    assert_eq!(recovered, out.crypto_key);
}
