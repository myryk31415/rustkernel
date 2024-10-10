#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustkernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustkernel::{
    allocator, hlt_loop, init,
    memory::{self},
    println,
};
use x86_64::VirtAddr;

entry_point!(kernel_main);

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

//entry point to the programm
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("main called!");

    init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { rustkernel::memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let x = Box::new(42123);
    println!("Box with value {} at address {:p}", x, x);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    #[cfg(test)]
    test_main();

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
