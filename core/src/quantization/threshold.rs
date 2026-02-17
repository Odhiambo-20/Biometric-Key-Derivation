use crate::error::Result;

pub fn quantize_threshold(embedding: &[f32], threshold: f32) -> Result<Vec<u8>> {
    Ok(embedding
        .iter()
        .map(|&v| if v >= threshold { 1u8 } else { 0u8 })
        .collect())
}

pub fn median_threshold(embedding: &[f32]) -> f32 {
    let mut v = embedding.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len().is_multiple_of(2) {
        (v[mid - 1] + v[mid]) * 0.5
    } else {
        v[mid]
    }
}
