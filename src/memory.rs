use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable
///
/// Complete physical memory must be mapped at offset
/// Function can only be called once
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_four_table = active_level_four_table(physical_memory_offset);
    OffsetPageTable::new(level_four_table, physical_memory_offset)
}

/// all physical memory need to be mapped
/// only call once
unsafe fn active_level_four_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_four_table_frame, _) = Cr3::read();

    let phys = level_four_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    // FIXME: This is not safe, only for testing
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };

    map_to_result.expect("map_to_failed").flush();
}

// pub struct EmptyFrameAllocator;

// unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
//         None
//     }
// }

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// returns usable frames from the bootloader's memory map
#[allow(dead_code)]
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

// use bootloader::bootinfo::MemoryRegion;

#[allow(dead_code)]
impl BootInfoFrameAllocator {
    /// memory map has to be valid. `USABLE` frames must be unused
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
