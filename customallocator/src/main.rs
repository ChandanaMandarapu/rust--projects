use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::cell::UnsafeCell;
use std::mem;

const ARENA_SIZE: usize = 1024 * 1024 * 4;
const SLAB_SIZES: [usize; 8] = [16, 32, 64, 128, 256, 512, 1024, 2048];
const MAX_SLAB_SIZE: usize = 2048;

struct ArenaAllocator {
    arenas: AtomicPtr<ArenaNode>,
    current_offset: AtomicUsize,
}

struct ArenaNode {
    data: *mut u8,
    size: usize,
    next: AtomicPtr<ArenaNode>,
}

impl ArenaNode {
    unsafe fn new(size: usize) -> *mut Self {
        let layout = Layout::from_size_align_unchecked(size, 16);
        let data = System.alloc(layout);
        
        if data.is_null() {
            return ptr::null_mut();
        }
        
        let node_layout = Layout::new::<ArenaNode>();
        let node = System.alloc(node_layout) as *mut ArenaNode;
        
        if node.is_null() {
            System.dealloc(data, layout);
            return ptr::null_mut();
        }
        
        ptr::write(node, ArenaNode {
            data,
            size,
            next: AtomicPtr::new(ptr::null_mut()),
        });
        
        node
    }
}

impl ArenaAllocator {
    const fn new() -> Self {
        ArenaAllocator {
            arenas: AtomicPtr::new(ptr::null_mut()),
            current_offset: AtomicUsize::new(0),
        }
    }

    unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        
        loop {
            let arena = self.arenas.load(Ordering::Acquire);
            
            if arena.is_null() {
                self.create_arena();
                continue;
            }
            
            let offset = self.current_offset.load(Ordering::Acquire);
            let aligned_offset = (offset + align - 1) & !(align - 1);
            
            if aligned_offset + size > (*arena).size {
                self.create_arena();
                continue;
            }
            
            if self.current_offset.compare_exchange(
                offset,
                aligned_offset + size,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                return (*arena).data.add(aligned_offset);
            }
        }
    }

    unsafe fn create_arena(&self) {
        let new_arena = ArenaNode::new(ARENA_SIZE);
        
        if new_arena.is_null() {
            return;
        }
        
        loop {
            let old_arena = self.arenas.load(Ordering::Acquire);
            (*new_arena).next.store(old_arena, Ordering::Release);
            
            if self.arenas.compare_exchange(
                old_arena,
                new_arena,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                self.current_offset.store(0, Ordering::Release);
                break;
            }
        }
    }
}

struct SlabAllocator {
    slabs: [AtomicPtr<SlabList>; 8],
}

struct SlabList {
    free_list: AtomicPtr<FreeNode>,
    chunk_size: usize,
    chunks_per_slab: usize,
}

struct FreeNode {
    next: AtomicPtr<FreeNode>,
}

struct SlabChunk {
    data: *mut u8,
    next: AtomicPtr<SlabChunk>,
}

impl SlabAllocator {
    const fn new() -> Self {
        SlabAllocator {
            slabs: [
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
            ],
        }
    }

    fn size_to_index(size: usize) -> Option<usize> {
        SLAB_SIZES.iter().position(|&s| s >= size)
    }

    unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        
        if size > MAX_SLAB_SIZE {
            return ptr::null_mut();
        }
        
        let idx = match Self::size_to_index(size) {
            Some(i) => i,
            None => return ptr::null_mut(),
        };
        
        let slab_list = self.get_or_create_slab_list(idx);
        
        if slab_list.is_null() {
            return ptr::null_mut();
        }
        
        loop {
            let free_node = (*slab_list).free_list.load(Ordering::Acquire);
            
            if free_node.is_null() {
                self.allocate_new_slab(slab_list);
                continue;
            }
            
            let next = (*free_node).next.load(Ordering::Acquire);
            
            if (*slab_list).free_list.compare_exchange(
                free_node,
                next,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                return free_node as *mut u8;
            }
        }
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        
        if size > MAX_SLAB_SIZE {
            return;
        }
        
        let idx = match Self::size_to_index(size) {
            Some(i) => i,
            None => return,
        };
        
        let slab_list = self.slabs[idx].load(Ordering::Acquire);
        
        if slab_list.is_null() {
            return;
        }
        
        let free_node = ptr as *mut FreeNode;
        
        loop {
            let old_head = (*slab_list).free_list.load(Ordering::Acquire);
            (*free_node).next.store(old_head, Ordering::Release);
            
            if (*slab_list).free_list.compare_exchange(
                old_head,
                free_node,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                break;
            }
        }
    }

    unsafe fn get_or_create_slab_list(&self, idx: usize) -> *mut SlabList {
        let slab_list = self.slabs[idx].load(Ordering::Acquire);
        
        if !slab_list.is_null() {
            return slab_list;
        }
        
        let chunk_size = SLAB_SIZES[idx];
        let chunks_per_slab = 4096 / chunk_size;
        
        let layout = Layout::new::<SlabList>();
        let new_slab_list = System.alloc(layout) as *mut SlabList;
        
        if new_slab_list.is_null() {
            return ptr::null_mut();
        }
        
        ptr::write(new_slab_list, SlabList {
            free_list: AtomicPtr::new(ptr::null_mut()),
            chunk_size,
            chunks_per_slab,
        });
        
        if self.slabs[idx].compare_exchange(
            ptr::null_mut(),
            new_slab_list,
            Ordering::Release,
            Ordering::Acquire,
        ).is_err() {
            System.dealloc(new_slab_list as *mut u8, layout);
            return self.slabs[idx].load(Ordering::Acquire);
        }
        
        new_slab_list
    }

    unsafe fn allocate_new_slab(&self, slab_list: *mut SlabList) {
        let chunk_size = (*slab_list).chunk_size;
        let chunks_per_slab = (*slab_list).chunks_per_slab;
        let total_size = chunk_size * chunks_per_slab;
        
        let layout = Layout::from_size_align_unchecked(total_size, 16);
        let slab_data = System.alloc(layout);
        
        if slab_data.is_null() {
            return;
        }
        
        for i in 0..chunks_per_slab {
            let chunk_ptr = slab_data.add(i * chunk_size) as *mut FreeNode;
            
            loop {
                let old_head = (*slab_list).free_list.load(Ordering::Acquire);
                (*chunk_ptr).next.store(old_head, Ordering::Release);
                
                if (*slab_list).free_list.compare_exchange(
                    old_head,
                    chunk_ptr,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    break;
                }
            }
        }
    }
}

struct HybridAllocator {
    arena: ArenaAllocator,
    slab: SlabAllocator,
    large_blocks: AtomicPtr<LargeBlockNode>,
}

struct LargeBlockNode {
    ptr: *mut u8,
    size: usize,
    layout: Layout,
    next: AtomicPtr<LargeBlockNode>,
}

impl HybridAllocator {
    const fn new() -> Self {
        HybridAllocator {
            arena: ArenaAllocator::new(),
            slab: SlabAllocator::new(),
            large_blocks: AtomicPtr::new(ptr::null_mut()),
        }
    }

    unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        
        if size <= MAX_SLAB_SIZE && layout.align() <= 16 {
            let ptr = self.slab.allocate(layout);
            if !ptr.is_null() {
                return ptr;
            }
        }
        
        if size <= 4096 {
            return self.arena.allocate(layout);
        }
        
        self.allocate_large(layout)
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        
        if size <= MAX_SLAB_SIZE && layout.align() <= 16 {
            self.slab.deallocate(ptr, layout);
        } else if size > 4096 {
            self.deallocate_large(ptr, layout);
        }
    }

    unsafe fn allocate_large(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        
        if ptr.is_null() {
            return ptr::null_mut();
        }
        
        let node_layout = Layout::new::<LargeBlockNode>();
        let node = System.alloc(node_layout) as *mut LargeBlockNode;
        
        if node.is_null() {
            System.dealloc(ptr, layout);
            return ptr::null_mut();
        }
        
        ptr::write(node, LargeBlockNode {
            ptr,
            size: layout.size(),
            layout,
            next: AtomicPtr::new(ptr::null_mut()),
        });
        
        loop {
            let old_head = self.large_blocks.load(Ordering::Acquire);
            (*node).next.store(old_head, Ordering::Release);
            
            if self.large_blocks.compare_exchange(
                old_head,
                node,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                break;
            }
        }
        
        ptr
    }

    unsafe fn deallocate_large(&self, ptr: *mut u8, layout: Layout) {
        let mut current = self.large_blocks.load(Ordering::Acquire);
        let mut prev: *mut LargeBlockNode = ptr::null_mut();
        
        while !current.is_null() {
            if (*current).ptr == ptr {
                let next = (*current).next.load(Ordering::Acquire);
                
                if prev.is_null() {
                    if self.large_blocks.compare_exchange(
                        current,
                        next,
                        Ordering::Release,
                        Ordering::Acquire,
                    ).is_ok() {
                        System.dealloc(ptr, (*current).layout);
                        let node_layout = Layout::new::<LargeBlockNode>();
                        System.dealloc(current as *mut u8, node_layout);
                        break;
                    }
                } else {
                    (*prev).next.store(next, Ordering::Release);
                    System.dealloc(ptr, (*current).layout);
                    let node_layout = Layout::new::<LargeBlockNode>();
                    System.dealloc(current as *mut u8, node_layout);
                    break;
                }
            }
            
            prev = current;
            current = (*current).next.load(Ordering::Acquire);
        }
    }
}

struct BuddyAllocator {
    orders: [AtomicPtr<BuddyBlock>; 12],
    base_ptr: *mut u8,
    total_size: usize,
}

struct BuddyBlock {
    next: AtomicPtr<BuddyBlock>,
    order: usize,
}

impl BuddyAllocator {
    unsafe fn new(size: usize) -> Self {
        let layout = Layout::from_size_align_unchecked(size, 4096);
        let base_ptr = System.alloc(layout);
        
        let mut allocator = BuddyAllocator {
            orders: [
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
            ],
            base_ptr,
            total_size: size,
        };
        
        let max_order = allocator.size_to_order(size);
        let initial_block = base_ptr as *mut BuddyBlock;
        ptr::write(initial_block, BuddyBlock {
            next: AtomicPtr::new(ptr::null_mut()),
            order: max_order,
        });
        allocator.orders[max_order].store(initial_block, Ordering::Release);
        
        allocator
    }

    fn size_to_order(&self, size: usize) -> usize {
        let mut order = 0;
        let mut block_size = 4096;
        
        while block_size < size && order < 11 {
            block_size *= 2;
            order += 1;
        }
        
        order
    }

    unsafe fn allocate(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(mem::size_of::<BuddyBlock>());
        let order = self.size_to_order(size);
        
        if order >= 12 {
            return ptr::null_mut();
        }
        
        for current_order in order..12 {
            loop {
                let block = self.orders[current_order].load(Ordering::Acquire);
                
                if block.is_null() {
                    break;
                }
                
                let next = (*block).next.load(Ordering::Acquire);
                
                if self.orders[current_order].compare_exchange(
                    block,
                    next,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    if current_order > order {
                        self.split_block(block, current_order, order);
                    }
                    return block as *mut u8;
                }
            }
        }
        
        ptr::null_mut()
    }

    unsafe fn split_block(&self, block: *mut BuddyBlock, from_order: usize, to_order: usize) {
        let mut current_order = from_order;
        let mut current_block = block;
        
        while current_order > to_order {
            current_order -= 1;
            let block_size = 4096 << current_order;
            let buddy = (current_block as *mut u8).add(block_size) as *mut BuddyBlock;
            
            ptr::write(buddy, BuddyBlock {
                next: AtomicPtr::new(ptr::null_mut()),
                order: current_order,
            });
            
            loop {
                let old_head = self.orders[current_order].load(Ordering::Acquire);
                (*buddy).next.store(old_head, Ordering::Release);
                
                if self.orders[current_order].compare_exchange(
                    old_head,
                    buddy,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    break;
                }
            }
        }
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(mem::size_of::<BuddyBlock>());
        let order = self.size_to_order(size);
        
        let block = ptr as *mut BuddyBlock;
        (*block).order = order;
        
        self.merge_and_free(block, order);
    }

    unsafe fn merge_and_free(&self, block: *mut BuddyBlock, order: usize) {
        let mut current_block = block;
        let mut current_order = order;
        
        while current_order < 11 {
            let block_size = 4096 << current_order;
            let offset = (current_block as usize) - (self.base_ptr as usize);
            let buddy_offset = offset ^ block_size;
            let buddy = (self.base_ptr as usize + buddy_offset) as *mut BuddyBlock;
            
            if !self.try_remove_from_free_list(buddy, current_order) {
                break;
            }
            
            current_block = if (current_block as usize) < (buddy as usize) {
                current_block
            } else {
                buddy
            };
            
            current_order += 1;
        }
        
        (*current_block).order = current_order;
        
        loop {
            let old_head = self.orders[current_order].load(Ordering::Acquire);
            (*current_block).next.store(old_head, Ordering::Release);
            
            if self.orders[current_order].compare_exchange(
                old_head,
                current_block,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                break;
            }
        }
    }

    unsafe fn try_remove_from_free_list(&self, block: *mut BuddyBlock, order: usize) -> bool {
        loop {
            let mut current = self.orders[order].load(Ordering::Acquire);
            let mut prev: *mut BuddyBlock = ptr::null_mut();
            
            while !current.is_null() {
                if current == block {
                    let next = (*current).next.load(Ordering::Acquire);
                    
                    if prev.is_null() {
                        if self.orders[order].compare_exchange(
                            current,
                            next,
                            Ordering::Release,
                            Ordering::Acquire,
                        ).is_ok() {
                            return true;
                        }
                        break;
                    } else {
                        (*prev).next.store(next, Ordering::Release);
                        return true;
                    }
                }
                
                prev = current;
                current = (*current).next.load(Ordering::Acquire);
            }
            
            if current.is_null() {
                return false;
            }
        }
    }
}

pub struct GlobalCustomAllocator;

static ALLOCATOR: HybridAllocator = HybridAllocator::new();

unsafe impl GlobalAlloc for GlobalCustomAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR.allocate(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.deallocate(ptr, layout)
    }
}