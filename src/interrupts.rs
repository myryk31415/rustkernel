use crate::{exit_qemu, gdt, hlt_loop, print, println, serial_print, serial_println};
use bitflags::{bitflags, Flags};
use core::arch::{asm, global_asm};
use core::default;
use core::fmt::Debug;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::{self, Mutex};
use x86_64::instructions::port::{Port, PortGeneric};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::{instructions::interrupts, registers::control};

pub mod idt;

#[repr(C)]
pub struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

impl Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InterruptStackFrame")
            .field(
                "instruction_pointer",
                &format_args!("virtual address - {:#x}", self.instruction_pointer),
            )
            .field("code_segment", &self.instruction_pointer)
            .field("cpu_flags", &format_args!("{:#x}", &self.cpu_flags))
            .field(
                "stack_pointer",
                &format_args!("virtual address - {:#x}", self.stack_pointer),
            )
            .field("stack_segment", &self.stack_segment)
            .finish()
    }
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

        // cpu exceptions
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, handler!(breakpoint_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(8, handler_with_error_code!(double_fault_handler)).
            set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        idt.set_handler(14, handler_with_error_code!(page_fault_handler));

        // hardware interrupts
        idt.set_handler(InterruptIndex::Timer.as_u8(), handler!(timer_interrupt_handler));
        idt.set_handler(InterruptIndex::Keyboard.as_u8(), handler!(keyboard_interrupt_handler));

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
    panic!("EXCEPTION: DOUBLE FAULT!\n{:#?}", stack_frame);
}

extern "C" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) -> () {
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:?}\n\
        error code: {:?}\n{:#?}",
        control::Cr2::read(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame
    );
    hlt_loop();
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

// hardware interrupt

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self as u8)
    }
}

extern "C" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "C" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port: Port<u8> = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

// test cases

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
