use crate::println;

pub static OS_LOGO_HEADER: &str = " ____                     ____    _____ 
|  _ \\                   / __ \\  / ____|
| |_) | _   _  ____ ____| |  | || (___  
|  _ < | | | ||_  /|_  /| |  | | \\___ \\ 
| |_) || |_| | / /  / / | |__| | ____) |
|____/  \\__,_|/___|/___| \\____/ |_____/\n
Version 0.1 - By Joao Kdouk\n\n";

pub fn print_logo() {
    println!("{}", OS_LOGO_HEADER);
}
