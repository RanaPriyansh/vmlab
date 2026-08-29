# vmlab - Virtual Memory Lab

A CS-spine memory management lab implemented in Rust. This project provides educational implementations of common memory allocation strategies and virtual memory concepts used in operating systems and high-performance computing.

## Features

- **Bump Allocator**: O(1) arena-style allocator with fast allocation and bulk reset
- **Freelist Allocator**: First-fit allocator with block coalescing, double-free detection, and fragmentation tracking
- **Page Table Simulation**: 4KB page-based virtual memory with address translation, page faults, and access tracking

## Quick Start

### Prerequisites

- Rust 1.56+ (edition 2021)
- Cargo

### Clone and Test

```bash
git clone https://github.com/RanaPriyansh/vmlab.git
cd vmlab
cargo test
```

### Build

```bash
cargo build --release
```

## Usage Examples

### Bump Allocator

```rust
use vmlab::BumpAllocator;

let mut allocator = BumpAllocator::new(1024);

// Allocate 64 bytes with 1-byte alignment
let offset = allocator.allocate(64, 1).unwrap();

// Write data
allocator.write(offset, b"Hello, World!").unwrap();

// Read data
let data = allocator.read(offset, 13).unwrap();
assert_eq!(data, b"Hello, World!");

// Reset to reuse all memory
allocator.reset();
```

### Freelist Allocator

```rust
use vmlab::FreelistAllocator;

let mut allocator = FreelistAllocator::new(4096);

// Allocate blocks
let a = allocator.allocate(256, 1).unwrap();
let b = allocator.allocate(512, 1).unwrap();

// Free a block (can be reused)
allocator.free(a, 256).unwrap();

// Allocate again (reuses freed space)
let c = allocator.allocate(128, 1).unwrap();

// Track fragmentation
println!("Fragmentation ratio: {:.2}", allocator.fragmentation_ratio());
println!("Free blocks: {}", allocator.free_block_count());
```

### Page Table

```rust
use vmlab::{PageTable, PAGE_SIZE};

let mut page_table = PageTable::new(16); // 16 physical pages

// Allocate virtual memory (4 pages)
let vaddr = page_table.vmalloc(4).unwrap();

// Translate virtual address to physical
let paddr = page_table.translate(vaddr + 100).unwrap();
println!("Virtual {} -> Physical {}", vaddr + 100, paddr);

// Mark page as dirty (for write operations)
page_table.mark_dirty(vaddr).unwrap();

// Check access/dirty bits
assert!(page_table.is_accessed(vaddr));
assert!(page_table.is_dirty(vaddr));

// Free virtual memory
page_table.vfree(vaddr, 4).unwrap();
```

## Testing

The project includes 42 comprehensive tests covering:

- Basic allocation and deallocation
- Edge cases (OOM, double-free, unaligned access)
- Memory coalescing
- Fragmentation tracking
- Virtual memory translation and page faults
- Integration scenarios combining allocators with page tables

Run all tests:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

## Architecture

### Bump Allocator
- **Strategy**: Sequential allocation by incrementing a pointer
- **Complexity**: O(1) allocation
- **Use Case**: Fast temporary allocations that are freed in bulk

### Freelist Allocator
- **Strategy**: First-fit with immediate coalescing
- **Features**: Double-free protection, fragmentation metrics
- **Use Case**: General-purpose allocation with reuse

### Page Table
- **Page Size**: 4KB (standard x86-64 page size)
- **Features**: Virtual-to-physical translation, page faults, access/dirty bits
- **Use Case**: Operating system virtual memory simulation

## Connection to Production Systems

See [NOTES.md](NOTES.md) for how these concepts map to real-world systems including:
- Operating system virtual memory
- Database buffer pools
- GPU memory management
- LLM inference systems (KV cache, PagedAttention)

## License

MIT License - see [LICENSE](LICENSE) for details

## Contributing

This is an educational project. Feel free to fork and experiment!
