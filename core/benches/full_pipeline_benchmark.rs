use std::time::Instant;

use biometric_core::{enroll, recover, BchParams, QuantizationMethod};

fn main() {
    let embedding: Vec<f32> = (0..512)
        .map(|i| if i % 3 == 0 { 0.6 } else { -0.4 })
        .collect();

    let params = BchParams::default();
    let iters = 1_000;

    let start = Instant::now();
    for _ in 0..iters {
        let out = enroll(&embedding, QuantizationMethod::Sign, params).unwrap();
        let _ = recover(&embedding, QuantizationMethod::Sign, &out.helper_data).unwrap();
    }
    let elapsed = start.elapsed();

    let per_op_us = elapsed.as_micros() as f64 / iters as f64;
    println!("full_pipeline_us_per_op={per_op_us:.3}");
}
