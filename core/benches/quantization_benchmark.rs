use std::time::Instant;

use biometric_core::quantization::{quantize_embedding, QuantizationMethod};

fn main() {
    let embedding: Vec<f32> = (0..512).map(|i| ((i as f32) / 256.0) - 1.0).collect();

    let iters = 5_000;
    let start = Instant::now();
    for _ in 0..iters {
        let _ = quantize_embedding(&embedding, QuantizationMethod::Sign).unwrap();
    }
    let elapsed = start.elapsed();

    let per_op_us = elapsed.as_micros() as f64 / iters as f64;
    println!("quantization_sign_us_per_op={per_op_us:.3}");
}
