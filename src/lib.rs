pub mod bump_allocator;
pub mod freelist_allocator;
pub mod page_table;

pub use bump_allocator::BumpAllocator;
pub use freelist_allocator::FreelistAllocator;
pub use page_table::{PageTable, PageTableEntry, PageFaultReason, PAGE_SIZE};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_bump_allocator_with_page_simulation() {
        let mut bump = BumpAllocator::new(PAGE_SIZE * 4);
        
        let page1 = bump.allocate(PAGE_SIZE, PAGE_SIZE).unwrap();
        let page2 = bump.allocate(PAGE_SIZE, PAGE_SIZE).unwrap();
        
        assert_eq!(page1, 0);
        assert_eq!(page2, PAGE_SIZE);
        
        bump.write(page1, &[1, 2, 3, 4]).unwrap();
        bump.write(page2, &[5, 6, 7, 8]).unwrap();
        
        assert_eq!(bump.read(page1, 4).unwrap(), &[1, 2, 3, 4]);
        assert_eq!(bump.read(page2, 4).unwrap(), &[5, 6, 7, 8]);
    }

    #[test]
    fn test_freelist_with_page_sized_allocations() {
        let mut freelist = FreelistAllocator::new(PAGE_SIZE * 8);
        
        let page1 = freelist.allocate(PAGE_SIZE, PAGE_SIZE).unwrap();
        let page2 = freelist.allocate(PAGE_SIZE, PAGE_SIZE).unwrap();
        let page3 = freelist.allocate(PAGE_SIZE, PAGE_SIZE).unwrap();
        
        assert_eq!(page1 % PAGE_SIZE, 0);
        assert_eq!(page2 % PAGE_SIZE, 0);
        assert_eq!(page3 % PAGE_SIZE, 0);
        
        freelist.free(page2, PAGE_SIZE).unwrap();
        
        let page4 = freelist.allocate(PAGE_SIZE, PAGE_SIZE).unwrap();
        assert_eq!(page4, page2);
    }

    #[test]
    fn test_page_table_with_allocator_simulation() {
        let mut pt = PageTable::new(16);
        let mut allocator = FreelistAllocator::new(16 * PAGE_SIZE);
        
        let vaddr1 = pt.vmalloc(2).unwrap();
        let phys1 = allocator.allocate(2 * PAGE_SIZE, PAGE_SIZE).unwrap();
        assert_eq!(phys1 % PAGE_SIZE, 0);
        
        let translated = pt.translate(vaddr1).unwrap();
        assert!(translated < 16 * PAGE_SIZE);
        
        assert!(pt.is_accessed(vaddr1));
    }

    #[test]
    fn test_multiple_allocator_strategies() {
        let bump_capacity = PAGE_SIZE * 4;
        let freelist_capacity = PAGE_SIZE * 4;
        
        let bump = BumpAllocator::new(bump_capacity);
        let mut freelist = FreelistAllocator::new(freelist_capacity);
        
        let bump_alloc = bump.allocate(PAGE_SIZE, 1).unwrap();
        let free_alloc = freelist.allocate(PAGE_SIZE, 1).unwrap();
        
        assert_eq!(bump_alloc, 0);
        assert_eq!(free_alloc, 0);
        
        freelist.free(free_alloc, PAGE_SIZE).unwrap();
        
        let reused = freelist.allocate(PAGE_SIZE, 1).unwrap();
        assert_eq!(reused, free_alloc);
    }

    #[test]
    fn test_virtual_memory_lifecycle() {
        let mut pt = PageTable::new(32);
        
        let region1 = pt.vmalloc(4).unwrap();
        let region2 = pt.vmalloc(4).unwrap();
        
        assert_eq!(pt.mapped_pages(), 8);
        
        pt.translate(region1).unwrap();
        pt.mark_dirty(region1).unwrap();
        
        assert!(pt.is_accessed(region1));
        assert!(pt.is_dirty(region1));
        
        pt.vfree(region1, 4).unwrap();
        assert_eq!(pt.mapped_pages(), 4);
        
        let result = pt.translate(region1);
        assert!(result.is_err());
        
        let translated2 = pt.translate(region2);
        assert!(translated2.is_ok());
    }

    #[test]
    fn test_kv_cache_style_allocation() {
        let mut pt = PageTable::new(256);
        let mut allocator = FreelistAllocator::new(256 * PAGE_SIZE);
        
        let batch_size = 8;
        let seq_length = 4;
        
        let vaddr = pt.vmalloc(batch_size * seq_length).unwrap();
        let phys = allocator.allocate(batch_size * seq_length * PAGE_SIZE, PAGE_SIZE);
        
        assert!(phys.is_some());
        assert_eq!(pt.mapped_pages(), batch_size * seq_length);
        
        for i in 0..batch_size {
            let token_vaddr = vaddr + i * seq_length * PAGE_SIZE;
            let result = pt.translate(token_vaddr);
            assert!(result.is_ok());
        }
    }
}
