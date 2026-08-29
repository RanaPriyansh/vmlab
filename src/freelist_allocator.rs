use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
    offset: usize,
    size: usize,
}

/// A freelist allocator using first-fit strategy with coalescing and double-free detection.
pub struct FreelistAllocator {
    capacity: usize,
    free_blocks: Vec<Block>,
    allocated_blocks: HashSet<usize>,
    total_allocated: usize,
}

impl FreelistAllocator {
    /// Creates a new freelist allocator with the given capacity in bytes
    pub fn new(capacity: usize) -> Self {
        let mut allocator = Self {
            capacity,
            free_blocks: Vec::new(),
            allocated_blocks: HashSet::new(),
            total_allocated: 0,
        };
        
        allocator.free_blocks.push(Block {
            offset: 0,
            size: capacity,
        });
        
        allocator
    }

    /// Allocates a block of the given size with alignment.
    /// Returns the offset, or None if allocation fails.
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<usize> {
        for i in 0..self.free_blocks.len() {
            let block = self.free_blocks[i];
            
            let aligned_offset = (block.offset + align - 1) & !(align - 1);
            let padding = aligned_offset - block.offset;
            
            if padding + size <= block.size {
                self.free_blocks.remove(i);
                
                if padding > 0 {
                    self.free_blocks.push(Block {
                        offset: block.offset,
                        size: padding,
                    });
                }
                
                let remaining = block.size - padding - size;
                if remaining > 0 {
                    self.free_blocks.push(Block {
                        offset: aligned_offset + size,
                        size: remaining,
                    });
                }
                
                self.allocated_blocks.insert(aligned_offset);
                self.total_allocated += size;
                self.sort_free_blocks();
                
                return Some(aligned_offset);
            }
        }
        
        None
    }

    /// Frees a previously allocated block at the given offset.
    /// Returns an error if the block was not allocated or already freed.
    pub fn free(&mut self, offset: usize, size: usize) -> Result<(), &'static str> {
        if !self.allocated_blocks.remove(&offset) {
            return Err("Double free or invalid offset");
        }
        
        self.total_allocated -= size;
        
        self.free_blocks.push(Block { offset, size });
        self.sort_free_blocks();
        self.coalesce();
        
        Ok(())
    }

    /// Sorts free blocks by offset
    fn sort_free_blocks(&mut self) {
        self.free_blocks.sort_by_key(|b| b.offset);
    }

    /// Coalesces adjacent free blocks
    fn coalesce(&mut self) {
        if self.free_blocks.len() <= 1 {
            return;
        }
        
        let mut coalesced = Vec::new();
        let mut current = self.free_blocks[0];
        
        for i in 1..self.free_blocks.len() {
            let next = self.free_blocks[i];
            
            if current.offset + current.size == next.offset {
                current.size += next.size;
            } else {
                coalesced.push(current);
                current = next;
            }
        }
        
        coalesced.push(current);
        self.free_blocks = coalesced;
    }

    /// Returns the total capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the amount of memory currently allocated
    pub fn allocated(&self) -> usize {
        self.total_allocated
    }

    /// Returns the amount of free memory
    pub fn free_memory(&self) -> usize {
        self.free_blocks.iter().map(|b| b.size).sum()
    }

    /// Returns the number of free blocks (fragmentation indicator)
    pub fn free_block_count(&self) -> usize {
        self.free_blocks.len()
    }

    /// Calculates fragmentation ratio (0.0 = no fragmentation, 1.0 = highly fragmented)
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.free_blocks.is_empty() {
            return 0.0;
        }
        
        let largest_free = self.free_blocks.iter().map(|b| b.size).max().unwrap_or(0);
        let total_free = self.free_memory();
        
        if total_free == 0 {
            return 0.0;
        }
        
        1.0 - (largest_free as f64 / total_free as f64)
    }

    /// Returns the largest free block size
    pub fn largest_free_block(&self) -> usize {
        self.free_blocks.iter().map(|b| b.size).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut allocator = FreelistAllocator::new(1024);
        let offset = allocator.allocate(64, 1).unwrap();
        assert_eq!(offset, 0);
        assert_eq!(allocator.allocated(), 64);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut allocator = FreelistAllocator::new(1024);
        let a = allocator.allocate(32, 1).unwrap();
        let b = allocator.allocate(64, 1).unwrap();
        let c = allocator.allocate(128, 1).unwrap();
        
        assert_eq!(a, 0);
        assert_eq!(b, 32);
        assert_eq!(c, 96);
        assert_eq!(allocator.allocated(), 224);
    }

    #[test]
    fn test_aligned_allocation() {
        let mut allocator = FreelistAllocator::new(1024);
        
        allocator.allocate(5, 1).unwrap();
        
        let aligned = allocator.allocate(16, 8).unwrap();
        assert_eq!(aligned % 8, 0);
    }

    #[test]
    fn test_free_and_reuse() {
        let mut allocator = FreelistAllocator::new(1024);
        
        let a = allocator.allocate(64, 1).unwrap();
        let _b = allocator.allocate(64, 1).unwrap();
        
        allocator.free(a, 64).unwrap();
        
        let c = allocator.allocate(32, 1).unwrap();
        assert_eq!(c, a);
    }

    #[test]
    fn test_double_free_detection() {
        let mut allocator = FreelistAllocator::new(1024);
        let offset = allocator.allocate(64, 1).unwrap();
        
        allocator.free(offset, 64).unwrap();
        
        let result = allocator.free(offset, 64);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Double free or invalid offset");
    }

    #[test]
    fn test_invalid_free() {
        let mut allocator = FreelistAllocator::new(1024);
        let result = allocator.free(500, 64);
        assert!(result.is_err());
    }

    #[test]
    fn test_coalescing_adjacent_blocks() {
        let mut allocator = FreelistAllocator::new(1024);
        
        let a = allocator.allocate(64, 1).unwrap();
        let b = allocator.allocate(64, 1).unwrap();
        let c = allocator.allocate(64, 1).unwrap();
        
        assert_eq!(allocator.free_block_count(), 1);
        
        allocator.free(a, 64).unwrap();
        assert_eq!(allocator.free_block_count(), 2);
        
        allocator.free(c, 64).unwrap();
        assert_eq!(allocator.free_block_count(), 2);
        
        allocator.free(b, 64).unwrap();
        assert_eq!(allocator.free_block_count(), 1);
    }

    #[test]
    fn test_fragmentation_tracking() {
        let mut allocator = FreelistAllocator::new(1024);
        
        let a = allocator.allocate(100, 1).unwrap();
        let _b = allocator.allocate(100, 1).unwrap();
        let c = allocator.allocate(100, 1).unwrap();
        let _d = allocator.allocate(100, 1).unwrap();
        
        allocator.free(a, 100).unwrap();
        allocator.free(c, 100).unwrap();
        
        assert!(allocator.free_block_count() > 1);
        assert!(allocator.fragmentation_ratio() > 0.0);
    }

    #[test]
    fn test_no_fragmentation_after_coalesce() {
        let mut allocator = FreelistAllocator::new(1024);
        
        let a = allocator.allocate(256, 1).unwrap();
        let b = allocator.allocate(256, 1).unwrap();
        
        allocator.free(a, 256).unwrap();
        allocator.free(b, 256).unwrap();
        
        assert_eq!(allocator.free_block_count(), 1);
        assert_eq!(allocator.fragmentation_ratio(), 0.0);
    }

    #[test]
    fn test_out_of_memory() {
        let mut allocator = FreelistAllocator::new(100);
        allocator.allocate(60, 1).unwrap();
        allocator.allocate(30, 1).unwrap();
        
        let result = allocator.allocate(20, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_largest_free_block() {
        let mut allocator = FreelistAllocator::new(1024);
        
        let a = allocator.allocate(200, 1).unwrap();
        let b = allocator.allocate(200, 1).unwrap();
        allocator.allocate(200, 1).unwrap();
        
        allocator.free(a, 200).unwrap();
        allocator.free(b, 200).unwrap();
        
        assert_eq!(allocator.largest_free_block(), 424);
    }

    #[test]
    fn test_capacity_tracking() {
        let mut allocator = FreelistAllocator::new(2048);
        assert_eq!(allocator.capacity(), 2048);
        assert_eq!(allocator.free_memory(), 2048);
        assert_eq!(allocator.allocated(), 0);
        
        allocator.allocate(512, 1).unwrap();
        assert_eq!(allocator.allocated(), 512);
        assert_eq!(allocator.free_memory(), 1536);
    }
}
