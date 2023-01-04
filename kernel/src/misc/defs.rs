#[repr(C)]
pub struct Stab {
    pub n_strx: u32,
    pub n_type: u8,
    pub n_other: u8,
    pub n_desc: u16,
    pub n_value: u32,
}
