#![no_std]
#![no_main]

#[allow(unused_imports)]
use user::libs::*;

#[no_mangle]
pub extern "C" fn _start() {
    system_call::exec("/sh");

    loop {}
}
