#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustkernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustkernel::{hlt_loop, init, println};

entry_point!(kernel_main);

//entry point to the programm
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("main called!");

    #[cfg(test)]
    test_main();

    init();

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("level 4 address: {:?}", level_4_page_table.start_address());

    // let ptr = 0x204013 as *mut u8;
    // unsafe { *ptr = 42 }
    // unsafe { *(0xdeadbeaf as *mut u8) = 42 };
    // x86_64::instructions::interrupts::int3();
    hlt_loop();
}

/// called on panic (no unwinding)
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
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
