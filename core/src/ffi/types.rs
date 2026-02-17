#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfiResultCode {
    pub code: i32,
}

impl FfiResultCode {
    pub const OK: Self = Self { code: 0 };
    pub const ERR_NULL: Self = Self { code: 1 };
    pub const ERR_INVALID: Self = Self { code: 2 };
}
