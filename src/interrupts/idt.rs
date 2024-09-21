use x86_64::registers::segmentation::{Segment, CS};
use x86_64::structures::gdt::SegmentSelector;
// use x86_64::structures::idt::HandlerFunc;
use x86_64::{PrivilegeLevel, VirtAddr};

pub type HandlerFunc = extern "C" fn() -> !;

pub struct Idt([Entry; 16]);

// interrupt descriptor table
impl Idt {
    pub fn new() -> Self {
        Idt([Entry::missing(); 16])
    }

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut EntryOptions {
        self.0[entry as usize] = Entry::new(CS::get_reg(), handler);
        // TODO
        &mut self.0[entry as usize].options
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        use x86_64::instructions::tables::{lidt, DescriptorTablePointer};

        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry {
    pointer_low: u16,
    gtd_selector: SegmentSelector,
    options: EntryOptions,
    pointer_mid: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Entry {
    fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Entry {
            gtd_selector: gdt_selector,
            options: EntryOptions::new(),
            pointer_low: pointer as u16,
            pointer_mid: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            reserved: 0,
        }
    }

    fn missing() -> Self {
        Entry {
            gtd_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            options: EntryOptions::minimal(),
            pointer_low: 0,
            pointer_mid: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn minimal() -> Self {
        let mut options = 0;
        options = set_bits(options, 9, 12, 0b111);
        EntryOptions(options)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0 = set_bits(self.0, 15, 16, present as u16);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0 = set_bits(self.0, 8, 9, !disable as u16);
        self
    }

    #[allow(dead_code)]
    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0 = set_bits(self.0, 13, 15, dpl);
        self
    }

    #[allow(dead_code)]
    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        // The hardware IST index starts at 1, but our software IST index
        // starts at 0. Therefore we need to add 1 here.
        // (thx to x86_64 crate)
        self.0 = set_bits(self.0, 0, 3, index + 1);
        self
    }
}

fn set_bits(mut valuee: u16, start: u16, end: u16, value: u16) -> u16 {
    let mut mask = 0;

    for _ in start..end {
        mask = mask << 1;
        mask += 1;
    }
    valuee &= !(mask << start);
    valuee | (value << start)
}

#[test_case]
fn test_set_bits() {
    let mut result = set_bits(0b0, 1, 2, 0b1);
    assert_eq!(result, 0b10);
    result = set_bits(0b100, 0, 2, 0b11);
    assert_eq!(result, 0b111);
    result = set_bits(0b11111111, 3, 7, 0b010);
    assert_eq!(result, 0b10010111);
    result = set_bits(0b1111111111111111, 0, 16, 0b0);
    assert_eq!(result, 0b0);
}
