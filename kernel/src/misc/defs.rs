#[repr(C)]
pub struct Stab {
    pub n_strx: usize,
    pub n_type: u8,
    pub n_other: u8,
    pub n_desc: u16,
    pub n_value: usize,
}

#[repr(C)]
pub struct StabInfo {
    pub eip_file: *const str,
    pub eip_line: usize,
    pub eip_fn_name: *const str,
    pub eip_fn_namelen: usize,
    pub eip_fn_addr: usize,
    pub eip_fn_narg: usize,
}
