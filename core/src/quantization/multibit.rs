use crate::error::Result;

pub fn quantize_2bit(embedding: &[f32]) -> Result<Vec<u8>> {
    // 2-bit scalar quantization in range [-1.0, 1.0].
    let mut out = Vec::with_capacity(embedding.len() * 2);
    for &v in embedding {
        let clamped = v.clamp(-1.0, 1.0);
        let bucket = if clamped < -0.5 {
            0u8
        } else if clamped < 0.0 {
            1u8
        } else if clamped < 0.5 {
            2u8
        } else {
            3u8
        };
        out.push((bucket >> 1) & 1);
        out.push(bucket & 1);
    }
    Ok(out)
}
