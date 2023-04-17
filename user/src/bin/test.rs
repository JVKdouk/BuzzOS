#![no_std]
#![no_main]

use core::panic::PanicInfo;

fn init_process() {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
