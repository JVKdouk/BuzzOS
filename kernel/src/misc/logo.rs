use crate::{devices::vga, devices::vga::Color::*, println, set_color};

pub static OS_LOGO_HEADER: &str = " ____                     ____    _____ 
|  _ \\                   / __ \\  / ____|
| |_) | _   _  ____ ____| |  | || (___  
|  _ < | | | ||_  /|_  /| |  | | \\___ \\ 
| |_) || |_| | / /  / / | |__| | ____) |
|____/  \\__,_|/___|/___| \\____/ |_____/";

pub fn print_logo() {
    set_color!(Yellow, Black);
    println!("{}", OS_LOGO_HEADER);

    set_color!(White, Black);
    println!("Version 0.1 - By Joao Kdouk\n\n");
}
