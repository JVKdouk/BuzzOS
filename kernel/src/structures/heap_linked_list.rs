/// Implementation of a heap-based linked list, allowing for growing and shrinking dynamically.
/// Can only be used after the heap has been setup.
use alloc::boxed::Box;

pub struct HeapLinkedList<T> {
    pub head: Option<Box<Node<T>>>,
    pub size: usize,
}

pub struct Node<T> {
    pub value: T,
    pub next: Option<Box<Node<T>>>
}

impl<T> HeapLinkedList<T> {
    pub const fn new() -> Self {
        HeapLinkedList {
            head: None,
            size: 0,
        }
    }

    pub fn push(&mut self, value: T) -> &Node<T> {
        let mut previous_head = core::mem::replace(&mut self.head, None);

        let node = Box::new(Node {
            value,
            next: previous_head
        });

        self.head = Some(node);
        self.size += 1;
        self.head.as_ref().unwrap()
    }

    pub fn pop(&mut self) -> Option<T> {
        match core::mem::replace(&mut self.head, None) {
            None => None,
            Some(node) => {
                self.size -= 1;
                self.head = node.next;
                Some(node.value)
            }
        }
    }

    pub fn peek(&self) -> Option<&Node<T>> {
        match self.head {
            None => None,
            Some(ref node) => Some(node.as_ref()),
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.head.is_none();
    }
}

impl<T> Drop for HeapLinkedList<T> {
    fn drop(&mut self) {
        let mut cur_link = core::mem::replace(&mut self.head, None);
        while let Some(mut boxed_node) = cur_link {
            cur_link = core::mem::replace(&mut boxed_node.next, None);
        }
    }
}
