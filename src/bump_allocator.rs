use std::cell::RefCell;

/// A simple bump allocator (arena allocator) that allocates memory in O(1) time
/// by incrementing a pointer. Memory can only be freed all at once via reset.
pub struct BumpAllocator {
    memory: Vec<u8>,
    offset: RefCell<usize>,
}

impl BumpAllocator {
    /// Creates a new bump allocator with the given capacity in bytes
    pub fn new(capacity: usize) -> Self {
        Self {
            memory: vec![0u8; capacity],
            offset: RefCell::new(0),
        }
    }

    /// Allocates `size` bytes with the given alignment.
    /// Returns the offset into the arena, or None if out of memory.
    pub fn allocate(&self, size: usize, align: usize) -> Option<usize> {
        let current = *self.offset.borrow();
        
        let align_offset = (current + align - 1) & !(align - 1);
        let new_offset = align_offset + size;
        
        if new_offset > self.memory.len() {
            return None;
        }
        
        *self.offset.borrow_mut() = new_offset;
        Some(align_offset)
    }

    /// Writes data at the given offset
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), &'static str> {
        if offset + data.len() > self.memory.len() {
            return Err("Write out of bounds");
        }
        self.memory[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Reads data from the given offset
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8], &'static str> {
        if offset + size > self.memory.len() {
            return Err("Read out of bounds");
        }
        Ok(&self.memory[offset..offset + size])
    }

    /// Resets the allocator, making all memory available again
    pub fn reset(&self) {
        *self.offset.borrow_mut() = 0;
    }

    /// Returns the current offset (amount of memory allocated)
    pub fn used(&self) -> usize {
        *self.offset.borrow()
    }

    /// Returns the total capacity
    pub fn capacity(&self) -> usize {
        self.memory.len()
    }

    /// Returns the amount of free memory
    pub fn available(&self) -> usize {
        self.capacity() - self.used()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let allocator = BumpAllocator::new(1024);
        let offset = allocator.allocate(64, 1).unwrap();
        assert_eq!(offset, 0);
        assert_eq!(allocator.used(), 64);
    }

    #[test]
    fn test_multiple_allocations() {
        let allocator = BumpAllocator::new(1024);
        let a = allocator.allocate(32, 1).unwrap();
        let b = allocator.allocate(64, 1).unwrap();
        let c = allocator.allocate(128, 1).unwrap();
        
        assert_eq!(a, 0);
        assert_eq!(b, 32);
        assert_eq!(c, 96);
        assert_eq!(allocator.used(), 224);
    }

    #[test]
    fn test_aligned_allocation() {
        let allocator = BumpAllocator::new(1024);
        
        allocator.allocate(5, 1).unwrap();
        
        let aligned = allocator.allocate(16, 8).unwrap();
        assert_eq!(aligned % 8, 0);
    }

    #[test]
    fn test_out_of_memory() {
        let allocator = BumpAllocator::new(100);
        allocator.allocate(60, 1).unwrap();
        allocator.allocate(30, 1).unwrap();
        
        let result = allocator.allocate(20, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_reset() {
        let allocator = BumpAllocator::new(1024);
        allocator.allocate(512, 1).unwrap();
        assert_eq!(allocator.used(), 512);
        
        allocator.reset();
        assert_eq!(allocator.used(), 0);
        
        let offset = allocator.allocate(256, 1).unwrap();
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_write_and_read() {
        let mut allocator = BumpAllocator::new(1024);
        let offset = allocator.allocate(32, 1).unwrap();
        
        let data = b"Hello, World!";
        allocator.write(offset, data).unwrap();
        
        let read_data = allocator.read(offset, data.len()).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_write_out_of_bounds() {
        let mut allocator = BumpAllocator::new(64);
        let result = allocator.write(60, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_out_of_bounds() {
        let allocator = BumpAllocator::new(64);
        let result = allocator.read(60, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_capacity_and_available() {
        let allocator = BumpAllocator::new(1024);
        assert_eq!(allocator.capacity(), 1024);
        assert_eq!(allocator.available(), 1024);
        
        allocator.allocate(256, 1).unwrap();
        assert_eq!(allocator.available(), 768);
    }
}
