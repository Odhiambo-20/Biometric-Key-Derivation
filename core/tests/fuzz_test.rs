use biometric_core::utils::hamming::hamming_distance;

#[test]
fn hamming_distance_basic() {
    let a = vec![0, 1, 1, 0, 1];
    let b = vec![0, 1, 0, 0, 1];
    let d = hamming_distance(&a, &b).unwrap();
    assert_eq!(d, 1);
}
