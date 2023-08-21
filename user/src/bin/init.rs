#![no_std]
#![no_main]

#[allow(unused_imports)]
use user::libs::*;

#[no_mangle]
pub extern "C" fn _start() {
    let fork_id = system_call::fork();

    if fork_id == 0 {
        system_call::exec("/sh");
    } else {
        system_call::wait();
    }

    loop {}
}
