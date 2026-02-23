use biometric_core::bch::{BchCodec, BchParams};

#[test]
fn bch_like_encode_decode_roundtrip() {
    let params = BchParams::new_2047_512(180);
    let codec = BchCodec::new(params);

    let data: Vec<u8> = (0..512).map(|i| (i % 2) as u8).collect();
    let codeword = codec.encode(&data).unwrap();
    assert_eq!(codeword.len(), 2047);

    let decoded = codec.decode(&codeword).unwrap();
    assert_eq!(decoded, data);
}
