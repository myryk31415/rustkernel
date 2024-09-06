#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustkernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustkernel::println;

#[no_mangle]
//entry point to the programm
pub extern "C" fn _start() -> ! {
    // panic!("an error occured!!!!");
    println!("main called!");
    for _ in 0..4 {
        println!("newline");
    }

    #[cfg(test)]
    test_main();

    loop {}
}

/// called on panic (no unwinding)
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustkernel::test_panic_handler(info)
}

#[test_case]
fn trivial_assertation() {
    assert_eq!(1, 1);
}
