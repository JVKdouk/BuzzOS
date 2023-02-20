/// Implementation of a heap-based linked list, allowing for growing and shrinking dynamically.
/// Can only be used after the heap has been setup.
use alloc::boxed::Box;

pub struct HeapLinkedList<T> {
    head: Option<Box<Node<T>>>,
}

pub struct Node<T> {
    pub value: T,
    pub next: Option<Box<Node<T>>>,
}

impl<T> HeapLinkedList<T> {
    pub const fn new() -> Self {
        HeapLinkedList { head: None }
    }

    pub fn push(&mut self, value: T) {
        let previous_head = core::mem::replace(&mut self.head, None);

        let node = Box::new(Node {
            value,
            next: previous_head,
        });

        self.head = Some(node);
    }

    pub fn push_node(&mut self, mut node: Box<Node<T>>) {
        let previous_head = core::mem::replace(&mut self.head, None);

        node.next = previous_head;

        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        match core::mem::replace(&mut self.head, None) {
            None => None,
            Some(node) => {
                self.head = node.next;
                Some(node.value)
            }
        }
    }

    pub fn pop_node(&mut self) -> Option<Box<Node<T>>> {
        match core::mem::replace(&mut self.head, None) {
            None => None,
            Some(mut node) => {
                core::mem::replace(&mut self.head, node.next.take());
                Some(node)
            }
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
