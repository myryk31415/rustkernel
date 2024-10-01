use x86_64::{structures::paging::{page_table, PageTable}, VirtAddr};


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
