#![no_std]
#![no_main]
#[allow(unused_imports)]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use user::libs::system_call::print_message;
use user::libs::*;

#[no_mangle]
pub extern "C" fn _start() {
    let a = Box::new(1).to_string();
    print_message(a.as_str());
    loop {}
}
