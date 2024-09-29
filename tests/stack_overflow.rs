#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::asm;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use rustkernel::interrupts::idt::Idt;
use rustkernel::interrupts::InterruptStackFrame;
use rustkernel::{exit_qemu, handler_with_error_code, serial_print, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rustkernel::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Double fault handler returned after stack overflow!");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustkernel::test_panic_handler(info);
}

lazy_static! {
    static ref TEST_IDT: Idt = {
        let mut idt = Idt::new();
        idt.set_handler(8, handler_with_error_code!(test_double_fault_handler))
            .set_stack_index(rustkernel::gdt::DOUBLE_FAULT_IST_INDEX);
        idt
    };
}

extern "C" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
}

pub fn init_test_idt() {
    TEST_IDT.load();
}
