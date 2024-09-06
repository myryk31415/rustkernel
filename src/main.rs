#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

#[no_mangle]
//entry point to the programm
pub extern "C" fn _start() -> ! {
    // panic!("an error occured!!!!");

    loop {}
}

//called on panic (no unwinding)
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    
    loop {}
}
