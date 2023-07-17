use alloc::alloc::{GlobalAlloc, Layout};

use crate::{
    memory::mem::MEMORY_REGION,
    println,
    structures::static_linked_list::StaticLinkedListNode,
    sync::spin_mutex::{SpinMutex, SpinMutexGuard},
    ROUND_UP,
};

use super::defs::{LinkedListAllocator, HEAP_PAGES, PAGE_SIZE};

pub struct Locked<A> {
    inner: SpinMutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: SpinMutex::new(inner),
        }
    }

    pub fn lock(&self) -> SpinMutexGuard<A> {
        self.inner.lock()
    }
}

#[global_allocator]
pub static HEAP_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
pub static IS_HEAP_ENABLED: SpinMutex<bool> = SpinMutex::new(false);

/// Heap Allocator is defined below. Rust no_std environment requires us
/// to define our own allocator. As such, our goal is to first identify what
/// memory region is available to be our Heap and then we instruct the allocator
/// on how to allocate memory in that region. For this implementation, a Linked
/// List allocator is used.
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(_layout);
        let mut allocator = self.lock();

        // Check if we can allocate a free node. If not, then we ran out of memory
        if let Some((node, start)) = allocator.search_free_node(size, align) {
            let end = start.checked_add(size).expect("overflow");
            let excess_size = node.end_address() - end;

            // If there is space left after we take the heap block, then this excess
            // node should be returned to the linked list
            if excess_size > 0 {
                allocator.add_free_node(end, excess_size);
            }

            start as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(_layout);
        self.lock().add_free_node(_ptr as usize, size);
    }
}

impl LinkedListAllocator {
    /// Create a new Linked List Allocator with zero allocated blocks
    /// Every Linked List starts with a zero-sized node, which marks the head.
    pub const fn new() -> Self {
        LinkedListAllocator {
            head: StaticLinkedListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, start_address: usize, size: usize) {
        self.add_free_node(start_address, size);
    }

    /// This allocator uses a Linked List to store memory nodes. If a memory region is free, it is
    /// added back to this Linked List. But notice, our Linked List is not dynamic, since it is not
    /// implemented using Rust's Box, then how do we do it?
    /// The answer is that we don't need the Box just yet, instead, we can use each freed memory block
    /// to store the Linked List Node information. If the block is allocated, Node data can be erased.
    pub unsafe fn add_free_node(&mut self, address: usize, size: usize) {
        // Ensures the provided address and size is aligned and capable of holding a list node
        assert_eq!(
            ROUND_UP!(address, core::mem::align_of::<StaticLinkedListNode>()),
            address
        );

        assert!(size >= core::mem::size_of::<StaticLinkedListNode>());

        // Create a new node to hold the memory region
        let mut node = StaticLinkedListNode::new(size);
        node.next = self.head.next.take(); // When we take, next becomes "None"

        // Update the given address with the new node information
        let node_address = address as *mut StaticLinkedListNode;
        node_address.write(node);

        self.head.next = Some(&mut *node_address);
    }

    pub fn allocate_free_node(
        node: &StaticLinkedListNode,
        size: usize,
        align: usize,
    ) -> Result<usize, ()> {
        let start = ROUND_UP!(node.address(), align);
        let end = start.checked_add(size).ok_or(())?;

        // Node not big enough
        if end > node.end_address() {
            return Err(());
        }

        let excess_size = node.end_address() - end;

        // The allocation procedure splits the region into a allocated region (of "size") and
        // a free region (of node.end - node.start - size). If the newly created free region
        // cannot contain Node information, then it cannot be tracked anymore by our allocator.
        // This would generate an unrecoverable memory leak over time.
        if excess_size > 0 && excess_size < core::mem::size_of::<StaticLinkedListNode>() {
            return Err(());
        }

        Ok(start)
    }

    /// Whenever allocation is requested, we must search for a suitable node. This procedure
    /// searches for a node that fits the requested memory size.
    pub fn search_free_node(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<(&'static mut StaticLinkedListNode, usize)> {
        // Start search at head
        let mut current_node = &mut self.head;

        // The idea is to iterate the linked list and try to find a suitable node.
        // For each iteration, we extract the next node, check if it is big enough
        // to house our newly allocated block, if it is, we return the new block,
        // else we keep trying, until we reach the end of the list, for which there
        // are no free blocks.
        while let Some(ref mut node) = current_node.next {
            if let Ok(start) = Self::allocate_free_node(&node, size, align) {
                // We found a block that fits, time to take the portion we want

                // Remove next node (the suitable one) and fix the Linked List
                let next = node.next.take();
                let ret = Some((current_node.next.take().unwrap(), start));
                current_node.next = next;
                return ret;
            } else {
                // Move to the next node
                current_node = current_node.next.as_mut().unwrap();
            }
        }

        // No nodes were found
        None
    }

    // Aligns the layout to the linked list node. This is necessary due to memory packing
    // since we want our layout to, after freed, also be able to house the linked list node.
    pub fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(core::mem::align_of::<StaticLinkedListNode>())
            .expect("[ERROR] Heap Alignment Failed")
            .pad_to_align();
        let size = layout
            .size()
            .max(core::mem::size_of::<StaticLinkedListNode>());
        return (size, layout.align());
    }
}

pub fn setup_heap() {
    println!("[KERNEL] Setting Up Heap");

    let page_address = MEMORY_REGION
        .lock()
        .next(HEAP_PAGES)
        .expect("[FATAL] Out of Memory")
        .address as usize;

    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(page_address, PAGE_SIZE * HEAP_PAGES);
    }

    *IS_HEAP_ENABLED.lock() = true;

    println!("[KERNEL] Allocated {} Heap Pages", HEAP_PAGES);
}
