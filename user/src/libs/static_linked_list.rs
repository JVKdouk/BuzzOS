pub struct StaticLinkedListNode {
    pub next: Option<&'static mut StaticLinkedListNode>,
    pub size: usize,
}

impl StaticLinkedListNode {
    pub const fn new(size: usize) -> Self {
        StaticLinkedListNode { next: None, size }
    }

    pub fn address(&self) -> usize {
        self as *const Self as usize
    }

    pub fn end_address(&self) -> usize {
        self.address() + self.size
    }
}
