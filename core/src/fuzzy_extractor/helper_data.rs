use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperData {
    pub version: u8,
    pub n: usize,
    pub k: usize,
    pub t: usize,
    pub helper_bits: Vec<u8>,
    pub commitment: [u8; 32],
    pub salt: [u8; 16],
}
