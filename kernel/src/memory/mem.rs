use crate::x86::helpers::stosb;

pub fn memset(address: usize, value: u8, length: usize) {
    stosb(address, value, length);
}
