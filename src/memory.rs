use x86_64::{
    instructions::tables,
    structures::paging::{frame, PageTable, OffsetPageTable},
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable
///
/// Complete physical memory must be mapped at offset
/// Function can only be called once
// pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
//     let level_four_table
//      = active_level_four_table(physical_memory_offset);
//     OffsetPageTable::new(level_four_table, physical_memory_offset)
// }

/// all physical memory need to be mapped
/// only call once
pub unsafe fn active_level_four_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_four_table_frame, _) = Cr3::read();

    let phys = level_four_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Translates a virtual address into a physical one
/// Returns `None` if not mapped
///
/// Whole physical memory needs to be mapped at given offset.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (level_four_table_frame, _) = Cr3::read();

    let tables_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_four_table_frame;

    for &index in &tables_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *mut PageTable = virt.as_mut_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Hugh pages not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
