#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfiResultCode {
    pub code: i32,
}

impl FfiResultCode {
    pub const OK: Self = Self { code: 0 };
    pub const ERR_NULL: Self = Self { code: 1 };
    pub const ERR_INVALID: Self = Self { code: 2 };
    pub const ERR_BUFFER_TOO_SMALL: Self = Self { code: 3 };
    pub const ERR_SERIALIZATION: Self = Self { code: 4 };
    pub const ERR_INTERNAL: Self = Self { code: 5 };
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BkdEnrollRequest {
    pub embedding_ptr: *const f32,
    pub embedding_len: usize,
    pub method: u32,
    pub threshold: f32,
    pub bch_t: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BkdEnrollResponse {
    pub helper_bits_out_ptr: *mut u8,
    pub helper_bits_out_cap: usize,
    pub salt_out_ptr: *mut u8,
    pub salt_out_cap: usize,
    pub commitment_out_ptr: *mut u8,
    pub commitment_out_cap: usize,
    pub key_out_ptr: *mut u8,
    pub key_out_cap: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BkdRecoverRequest {
    pub embedding_ptr: *const f32,
    pub embedding_len: usize,
    pub method: u32,
    pub threshold: f32,
    pub bch_t: usize,
    pub helper_bits_ptr: *const u8,
    pub helper_bits_len: usize,
    pub salt_ptr: *const u8,
    pub salt_len: usize,
    pub commitment_ptr: *const u8,
    pub commitment_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BkdRecoverResponse {
    pub key_out_ptr: *mut u8,
    pub key_out_cap: usize,
}
