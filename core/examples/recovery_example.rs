use biometric_core::{enroll, recover, BchParams, QuantizationMethod};

fn main() {
    let embedding: Vec<f32> = (0..128).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();

    let enrolled = enroll(&embedding, QuantizationMethod::Sign, BchParams::default()).unwrap();
    let recovered = recover(&embedding, QuantizationMethod::Sign, &enrolled.helper_data).unwrap();

    println!("key match: {}", recovered == enrolled.crypto_key);
}
