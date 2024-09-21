use core::arch::{asm, global_asm};

use crate::{gdt, println, serial_print, serial_println};
use bitflags::{bitflags, Flags};
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::{instructions::interrupts, registers::control};

pub mod idt;

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

pub fn init_idt() {
    IDT.load();
}

macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm! {  "
                push rax;
                push rcx;
                push rdx;
                push rsi;
                push rdi;
                push r8;
                push r9;
                push r10;
                push r11;
                mov rdi, rsp;
                add rdi, 9*8;
                call {};
                pop r11;
                pop r10;
                pop r9;
                pop r8;
                pop rdi;
                pop rsi;
                pop rdx;
                pop rcx;
                pop rax;
                iretq",
                sym $name, options(noreturn) };
            }
        }
        wrapper
    }};
}

#[macro_export]
macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm! {  "
                push rax;
                push rcx;
                push rdx;
                push rsi;
                push rdi;
                push r8;
                push r9;
                push r10;
                push r11;
                mov rsi, [rsp + 9*8] // get error code
                mov rdi, rsp;
                add rdi, 10*8; // execption stack frame pointer
                sub rsp, 8; // align stack pointer (stack frame + error code = aligned; 9*8 = 8 missing)
                call {};
                add rsp, 8; // undo alignment
                pop r11;
                pop r10;
                pop r9;
                pop r8;
                pop rdi;
                pop rsi;
                pop rdx;
                pop rcx;
                pop rax;
                add rsp, 8; // remove error code
                iretq",
                sym $name, options(noreturn) };
            }
        }
        wrapper
    }};
}

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, handler!(breakpoint_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(8, handler_with_error_code!(double_fault_handler)).
            set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        // idt.0[8].options.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        idt.set_handler(14, handler_with_error_code!(page_fault_handler));

        idt
    };
}

bitflags! {
    #[derive(Debug)]
    struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

// use core::fmt::Formatter;
// impl core::fmt::Debug for PageFaultErrorCode {
//     fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
//         self.iter_names().for_each(|f| println!("{}", f.0));
//         Ok(())
//     }
// }

extern "C" fn double_fault_handler(stack_frame: &InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCPETION: DOUBLE FAULT!\n{:#?}", stack_frame);
}

extern "C" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) -> () {
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#x}\n\
        error code: {:?}\n{:#?}",
        control::Cr2::read_raw(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        &*stack_frame
    );
}

extern "C" fn divide_by_zero_handler(stack_frame: &InterruptStackFrame) -> ! {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", &*stack_frame);
    loop {}
}

extern "C" fn breakpoint_handler(stack_frame: &InterruptStackFrame) {
    println!(
        "\nEXCEPTION: BREAKPOINT at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, &*stack_frame
    );
}

extern "C" fn invalid_opcode_handler(stack_frame: &InterruptStackFrame) -> ! {
    println!(
        "\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, &*stack_frame
    );
    loop {}
}

#[test_case]
fn test_breakpoint_exception() {
    init_idt();
    x86_64::instructions::interrupts::int3();
}

// #[test_case]
// fn test_page_fault_exception() {
//     init_idt();
//     unsafe { *(0xdeadbea0 as *mut u64) = 42 };
// }

