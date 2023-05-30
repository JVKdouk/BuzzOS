pub struct StaticLinkedListNode {
    pub next: Option<&'static mut StaticLinkedListNode>, // Reference to the next node in the list
    pub size: usize,                                     // Size of the current node
}

/// Static Linked List Node is used to build a Linked List that can be resolved at compile time.
/// This is useful in the Heap Allocator design, since at this point we do not have access to the
/// heap to create a dynamic linked list.
impl StaticLinkedListNode {
    /// Const functions can be evaluated at compile time, being more limited than other functions.
    /// This allows our function to be called in a static environment
    pub const fn new(size: usize) -> Self {
        StaticLinkedListNode { next: None, size }
    }

    /// Address in memory where this node starts
    pub fn address(&self) -> usize {
        self as *const Self as usize
    }

    /// Address in memory where this node ends
    pub fn end_address(&self) -> usize {
        self.address() + self.size
    }
}
