use biometric_core::quantization::multibit::quantize_2bit;
use biometric_core::quantization::sign_based::quantize_sign;
use biometric_core::quantization::threshold::quantize_threshold;

#[test]
fn sign_quantization_works() {
    let emb = vec![-1.0, 0.0, 0.5, -0.1];
    let bits = quantize_sign(&emb).unwrap();
    assert_eq!(bits, vec![0, 1, 1, 0]);
}

#[test]
fn threshold_quantization_works() {
    let emb = vec![-0.2, 0.1, 0.5, 0.0];
    let bits = quantize_threshold(&emb, 0.1).unwrap();
    assert_eq!(bits, vec![0, 1, 1, 0]);
}

#[test]
fn multibit_quantization_works() {
    let emb = vec![-1.0, -0.25, 0.25, 1.0];
    let bits = quantize_2bit(&emb).unwrap();
    assert_eq!(bits.len(), 8);
}
