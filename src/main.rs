#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustkernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustkernel::{
    hlt_loop, init,
    memory::{self},
    println,
};
use x86_64::{
    structures::paging::{Page, Size4KiB},
    VirtAddr,
};

entry_point!(kernel_main);

//entry point to the programm
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("main called!");

    #[cfg(test)]
    test_main();

    init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { rustkernel::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xdeadbeaf));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {
        page_ptr.offset(80).write_volatile(0x_f021_f077_f065_f04e);
    };
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
