pub fn decode_repetition_255_to_128(noisy_codeword: &[u8]) -> Vec<u8> {
    noisy_codeword.iter().copied().take(128).collect()
}
