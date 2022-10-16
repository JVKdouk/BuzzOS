use crate::print;
use crate::kernel::vga;
use core::fmt::Arguments;

pub fn logo() {
    print!(" ____                   ____   _____ 
|  _ \\                 / __ \\ / ____|
| |_) | ___  __ _ _ __| |  | | (___  
|  _ < / _ \\/ _` | '__| |  | |\\___ \\ 
| |_) |  __/ (_| | |  | |__| |____) |
|____/ \\___|\\__,_|_|   \\____/|_____/\n");
    print!("Version 0.1 - By Joao Kdouk\n\n");
}