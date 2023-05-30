use crate::x86::helpers::outb;

use self::{
    io_apic::{check_apic, setup_io_apic},
    local_apic::setup_local_apic,
    mp::setup_mp,
};

pub mod defs;
pub mod io_apic;
pub mod local_apic;
pub mod mp;

pub fn setup_apic() {
    check_apic();
    setup_mp();
    disable_pic();
    setup_local_apic();
    setup_io_apic();
}

pub fn disable_pic() {
    outb(0x21, 0xFF);
    outb(0xA1, 0xFF);
}
