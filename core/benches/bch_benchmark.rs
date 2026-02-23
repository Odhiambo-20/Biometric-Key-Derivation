use std::time::Instant;

use biometric_core::bch::{BchCodec, BchParams};

fn main() {
    let params = BchParams::default();
    let codec = BchCodec::new(params);

    let bits: Vec<u8> = (0..512).map(|i| (i % 2) as u8).collect();

    let iters = 1_000;

    let start_encode = Instant::now();
    let mut codeword = Vec::new();
    for _ in 0..iters {
        codeword = codec.encode(&bits).unwrap();
    }
    let encode_elapsed = start_encode.elapsed();

    let start_decode = Instant::now();
    for _ in 0..iters {
        let _ = codec.decode(&codeword).unwrap();
    }
    let decode_elapsed = start_decode.elapsed();

    let enc_us = encode_elapsed.as_micros() as f64 / iters as f64;
    let dec_us = decode_elapsed.as_micros() as f64 / iters as f64;

    println!("bch_encode_us_per_op={enc_us:.3}");
    println!("bch_decode_us_per_op={dec_us:.3}");
}
