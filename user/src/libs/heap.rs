use super::{
    spin_mutex::{SpinMutex, SpinMutexGuard},
    system_call::print_message,
};
use crate::{libs::static_linked_list::StaticLinkedListNode, ROUND_UP};
use alloc::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::AtomicBool;

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

pub struct LinkedListAllocator {
    head: StaticLinkedListNode,
}

#[global_allocator]
pub static HEAP_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
pub static IS_HEAP_ENABLED: AtomicBool = AtomicBool::new(false);

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(_layout);
        let mut allocator = self.lock();

        loop {
            if let Some((node, start)) = allocator.search_free_node(size, align) {
                // A sizeable heap fragment has been found
                let end = start.checked_add(size).expect("overflow");
                let excess_size = node.end_address() - end;

                if excess_size > 0 {
                    allocator.add_free_node(end, excess_size);
                }

                return start as *mut u8;
            } else {
                // No fragment fits the requirement, needs to ask the OS to increase
                // available memory. OS can fail the allocation.
                let current_end_address = super::system_call::sbrk(4096);
                if current_end_address == -1 {
                    return core::ptr::null_mut();
                }

                print_message("Allocating more memory");
                allocator.add_free_node(current_end_address as usize, 4096);
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(_layout);
        self.lock().add_free_node(_ptr as usize, size);
    }
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            head: StaticLinkedListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, start_address: usize, size: usize) {
        self.add_free_node(start_address, size);
    }

    pub unsafe fn add_free_node(&mut self, address: usize, size: usize) {
        assert_eq!(
            ROUND_UP!(address, core::mem::align_of::<StaticLinkedListNode>()),
            address
        );

        assert!(size >= core::mem::size_of::<StaticLinkedListNode>());

        let mut node = StaticLinkedListNode::new(size);
        node.next = self.head.next.take();

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

        if end > node.end_address() {
            return Err(());
        }

        let excess_size = node.end_address() - end;
        if excess_size > 0 && excess_size < core::mem::size_of::<StaticLinkedListNode>() {
            return Err(());
        }

        Ok(start)
    }

    pub fn search_free_node(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<(&'static mut StaticLinkedListNode, usize)> {
        let mut current_node = &mut self.head;

        while let Some(ref mut node) = current_node.next {
            if let Ok(start) = Self::allocate_free_node(&node, size, align) {
                let next = node.next.take();
                let ret = Some((current_node.next.take().unwrap(), start));
                current_node.next = next;
                return ret;
            } else {
                current_node = current_node.next.as_mut().unwrap();
            }
        }

        None
    }

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
    unsafe { HEAP_ALLOCATOR.lock().init(0, 0) };
}
