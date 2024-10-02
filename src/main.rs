#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustkernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustkernel::{hlt_loop, init, memory::active_level_four_table, println};
use x86_64::{structures::paging::PageTable, VirtAddr};

entry_point!(kernel_main);

//entry point to the programm
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rustkernel::memory::translate_addr;

    println!("main called!");

    #[cfg(test)]
    test_main();

    init();

    // use x86_64::registers::control::Cr3;

    // let (level_4_page_table, _) = Cr3::read();
    // println!("level 4 address: {:?}", level_4_page_table.start_address());

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_four_table(phys_mem_offset) };

    let addresses = [
        // identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // (phys_mem_offset + 0x401008 as u64).as_u64(),cr
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys_mem_offset + phys.as_u64();
    //         let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    //         let l3_table = unsafe { &*page_table_ptr };

    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!("L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

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
