use std::collections::HashMap;

pub const PAGE_SIZE: usize = 4096; // 4KB pages

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry {
    physical_page: usize,
    present: bool,
    accessed: bool,
    dirty: bool,
}

impl PageTableEntry {
    fn new(physical_page: usize) -> Self {
        Self {
            physical_page,
            present: true,
            accessed: false,
            dirty: false,
        }
    }
}

/// A simple page table simulation for virtual memory management.
/// Simulates a single-level page table with 4KB pages.
pub struct PageTable {
    entries: HashMap<usize, PageTableEntry>,
    next_physical_page: usize,
    max_physical_pages: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PageFaultReason {
    NotMapped,
    NotPresent,
}

impl PageTable {
    /// Creates a new page table with the given number of physical pages
    pub fn new(max_physical_pages: usize) -> Self {
        Self {
            entries: HashMap::new(),
            next_physical_page: 0,
            max_physical_pages,
        }
    }

    /// Translates a virtual address to a physical address.
    /// Returns the physical address or a page fault error.
    pub fn translate(&mut self, virtual_addr: usize) -> Result<usize, PageFaultReason> {
        let virtual_page = virtual_addr / PAGE_SIZE;
        let offset = virtual_addr % PAGE_SIZE;
        
        match self.entries.get_mut(&virtual_page) {
            Some(entry) if entry.present => {
                entry.accessed = true;
                Ok(entry.physical_page * PAGE_SIZE + offset)
            }
            Some(_) => Err(PageFaultReason::NotPresent),
            None => Err(PageFaultReason::NotMapped),
        }
    }

    /// Allocates and maps a range of virtual pages.
    /// Returns the starting virtual address or None if out of physical memory.
    pub fn vmalloc(&mut self, num_pages: usize) -> Option<usize> {
        if self.next_physical_page + num_pages > self.max_physical_pages {
            return None;
        }
        
        let start_virtual_page = self.find_free_virtual_range(num_pages)?;
        
        for i in 0..num_pages {
            let virtual_page = start_virtual_page + i;
            let physical_page = self.next_physical_page + i;
            
            self.entries.insert(
                virtual_page,
                PageTableEntry::new(physical_page),
            );
        }
        
        self.next_physical_page += num_pages;
        
        Some(start_virtual_page * PAGE_SIZE)
    }

    /// Frees a range of virtual pages.
    pub fn vfree(&mut self, virtual_addr: usize, num_pages: usize) -> Result<(), &'static str> {
        let start_virtual_page = virtual_addr / PAGE_SIZE;
        
        if virtual_addr % PAGE_SIZE != 0 {
            return Err("Virtual address must be page-aligned");
        }
        
        for i in 0..num_pages {
            let virtual_page = start_virtual_page + i;
            
            if !self.entries.contains_key(&virtual_page) {
                return Err("Attempting to free unmapped page");
            }
            
            self.entries.remove(&virtual_page);
        }
        
        Ok(())
    }

    /// Maps a specific virtual page to a physical page
    pub fn map(&mut self, virtual_page: usize, physical_page: usize) -> Result<(), &'static str> {
        if physical_page >= self.max_physical_pages {
            return Err("Physical page out of bounds");
        }
        
        if self.entries.contains_key(&virtual_page) {
            return Err("Virtual page already mapped");
        }
        
        self.entries.insert(virtual_page, PageTableEntry::new(physical_page));
        Ok(())
    }

    /// Unmaps a virtual page
    pub fn unmap(&mut self, virtual_page: usize) -> Result<(), &'static str> {
        if self.entries.remove(&virtual_page).is_none() {
            return Err("Virtual page not mapped");
        }
        Ok(())
    }

    /// Marks a page as accessed (for translate) or dirty (for write)
    pub fn mark_dirty(&mut self, virtual_addr: usize) -> Result<(), &'static str> {
        let virtual_page = virtual_addr / PAGE_SIZE;
        
        match self.entries.get_mut(&virtual_page) {
            Some(entry) => {
                entry.dirty = true;
                Ok(())
            }
            None => Err("Page not mapped"),
        }
    }

    /// Checks if a page is dirty
    pub fn is_dirty(&self, virtual_addr: usize) -> bool {
        let virtual_page = virtual_addr / PAGE_SIZE;
        self.entries.get(&virtual_page).map_or(false, |e| e.dirty)
    }

    /// Checks if a page was accessed
    pub fn is_accessed(&self, virtual_addr: usize) -> bool {
        let virtual_page = virtual_addr / PAGE_SIZE;
        self.entries.get(&virtual_page).map_or(false, |e| e.accessed)
    }

    /// Clears the accessed bit for a page
    pub fn clear_accessed(&mut self, virtual_addr: usize) -> Result<(), &'static str> {
        let virtual_page = virtual_addr / PAGE_SIZE;
        
        match self.entries.get_mut(&virtual_page) {
            Some(entry) => {
                entry.accessed = false;
                Ok(())
            }
            None => Err("Page not mapped"),
        }
    }

    /// Returns the number of mapped pages
    pub fn mapped_pages(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of allocated physical pages
    pub fn allocated_physical_pages(&self) -> usize {
        self.next_physical_page
    }

    /// Finds a contiguous range of free virtual pages
    fn find_free_virtual_range(&self, num_pages: usize) -> Option<usize> {
        let mut start_candidate = 0;
        let mut consecutive_free = 0;
        
        for candidate in 0..1_000_000 {
            if !self.entries.contains_key(&candidate) {
                if consecutive_free == 0 {
                    start_candidate = candidate;
                }
                consecutive_free += 1;
                if consecutive_free == num_pages {
                    return Some(start_candidate);
                }
            } else {
                consecutive_free = 0;
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_translation() {
        let mut pt = PageTable::new(16);
        
        pt.map(0, 0).unwrap();
        
        let phys = pt.translate(100).unwrap();
        assert_eq!(phys, 100);
        
        assert!(pt.is_accessed(100));
    }

    #[test]
    fn test_page_fault_not_mapped() {
        let mut pt = PageTable::new(16);
        
        let result = pt.translate(0);
        assert_eq!(result, Err(PageFaultReason::NotMapped));
    }

    #[test]
    fn test_vmalloc_basic() {
        let mut pt = PageTable::new(16);
        
        let vaddr = pt.vmalloc(4).unwrap();
        assert_eq!(vaddr % PAGE_SIZE, 0);
        
        let phys = pt.translate(vaddr).unwrap();
        assert!(phys < 16 * PAGE_SIZE);
    }

    #[test]
    fn test_vmalloc_multiple() {
        let mut pt = PageTable::new(16);
        
        let a = pt.vmalloc(2).unwrap();
        let b = pt.vmalloc(3).unwrap();
        let c = pt.vmalloc(1).unwrap();
        
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn test_vmalloc_out_of_memory() {
        let mut pt = PageTable::new(8);
        
        pt.vmalloc(5).unwrap();
        pt.vmalloc(2).unwrap();
        
        let result = pt.vmalloc(2);
        assert!(result.is_none());
    }

    #[test]
    fn test_vfree() {
        let mut pt = PageTable::new(16);
        
        let vaddr = pt.vmalloc(3).unwrap();
        assert_eq!(pt.mapped_pages(), 3);
        
        pt.vfree(vaddr, 3).unwrap();
        assert_eq!(pt.mapped_pages(), 0);
    }

    #[test]
    fn test_vfree_unmapped_error() {
        let mut pt = PageTable::new(16);
        let result = pt.vfree(0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_vfree_unaligned_error() {
        let mut pt = PageTable::new(16);
        let result = pt.vfree(100, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_and_unmap() {
        let mut pt = PageTable::new(16);
        
        pt.map(5, 3).unwrap();
        assert_eq!(pt.mapped_pages(), 1);
        
        let phys = pt.translate(5 * PAGE_SIZE + 200).unwrap();
        assert_eq!(phys, 3 * PAGE_SIZE + 200);
        
        pt.unmap(5).unwrap();
        assert_eq!(pt.mapped_pages(), 0);
    }

    #[test]
    fn test_dirty_bit() {
        let mut pt = PageTable::new(16);
        let vaddr = pt.vmalloc(1).unwrap();
        
        assert!(!pt.is_dirty(vaddr));
        
        pt.mark_dirty(vaddr).unwrap();
        assert!(pt.is_dirty(vaddr));
    }

    #[test]
    fn test_accessed_bit() {
        let mut pt = PageTable::new(16);
        let vaddr = pt.vmalloc(1).unwrap();
        
        pt.clear_accessed(vaddr).unwrap();
        assert!(!pt.is_accessed(vaddr));
        
        pt.translate(vaddr).unwrap();
        assert!(pt.is_accessed(vaddr));
        
        pt.clear_accessed(vaddr).unwrap();
        assert!(!pt.is_accessed(vaddr));
    }

    #[test]
    fn test_page_offset_translation() {
        let mut pt = PageTable::new(16);
        
        pt.map(10, 5).unwrap();
        
        let vaddr = 10 * PAGE_SIZE + 1234;
        let phys = pt.translate(vaddr).unwrap();
        assert_eq!(phys, 5 * PAGE_SIZE + 1234);
    }

    #[test]
    fn test_allocated_physical_pages() {
        let mut pt = PageTable::new(16);
        
        assert_eq!(pt.allocated_physical_pages(), 0);
        
        pt.vmalloc(3).unwrap();
        assert_eq!(pt.allocated_physical_pages(), 3);
        
        pt.vmalloc(2).unwrap();
        assert_eq!(pt.allocated_physical_pages(), 5);
    }

    #[test]
    fn test_map_already_mapped_error() {
        let mut pt = PageTable::new(16);
        pt.map(0, 0).unwrap();
        
        let result = pt.map(0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_physical_out_of_bounds() {
        let mut pt = PageTable::new(8);
        let result = pt.map(0, 10);
        assert!(result.is_err());
    }
}
