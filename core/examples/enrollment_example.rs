use biometric_core::{enroll, BchParams, QuantizationMethod};

fn main() {
    let embedding: Vec<f32> = (0..128).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();
    let out = enroll(&embedding, QuantizationMethod::Sign, BchParams::default()).unwrap();
    println!("helper bits: {}", out.helper_data.helper_bits.len());
    println!("key len: {}", out.crypto_key.len());
}
