use zeroize::Zeroize;

pub fn zeroize_vec(data: &mut Vec<u8>) {
    data.zeroize();
}

pub fn zeroize_f32_vec(data: &mut Vec<f32>) {
    data.zeroize();
}
