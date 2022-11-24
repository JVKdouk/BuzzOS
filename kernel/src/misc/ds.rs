use core::fmt::Debug;
use core::ptr;

use crate::println;

#[derive(Clone, Copy)]
pub struct LinkedListNode<T> {
    pub value: T,
    next: Option<*const LinkedListNode<T>>,
}

pub struct LinkedList<T> {
    pub head: Option<*const LinkedListNode<T>>,
    pub size: u64,
}

impl<T: Copy + Debug> LinkedList<T> {
    pub const fn new() -> LinkedList<T> {
        LinkedList {
            head: None,
            size: 0,
        }
    }

    pub fn push(&mut self, value: T) -> *const LinkedListNode<T> {
        if (self.head.is_none()) {
            self.head = Some(&LinkedListNode::new(value, None));
            return self.head.unwrap();
        }

        self.size += 1;

        let previous_node_value = match self.head {
            None => None,
            Some(x) => Some(unsafe { (*x).value }),
        };

        let new_node = LinkedListNode {
            value,
            next: self.head,
        };

        println!(
            "Head id {:p} with value {:X?}, previous {:X?}",
            &self.head,
            value,
            previous_node_value.unwrap()
        );
        self.head = Some(&new_node);
        return self.head.unwrap();
    }

    pub fn pop(&mut self) -> Option<T> {
        if (self.size == 0) {
            return None;
        }

        let head = unsafe { *self.head.unwrap() };

        if (self.size == 1) {
            let node = head;
            self.head = None;
            return Some(node.value);
        }

        self.size -= 1;

        let next = Some(head.get_next());
        self.head = next;
        return Some(head.value);
    }
}

impl<T: Copy> LinkedListNode<T> {
    pub const fn new(value: T, next: Option<*const LinkedListNode<T>>) -> LinkedListNode<T> {
        LinkedListNode { value, next }
    }

    pub fn set_next(&mut self, node: &mut LinkedListNode<T>) {
        self.next = Some(node as *const LinkedListNode<T>);
    }

    pub fn add_before(&mut self, mut node: LinkedListNode<T>) -> LinkedListNode<T> {
        node.set_next(self);
        return node;
    }

    pub fn add_value_before(&mut self, value: T) -> LinkedListNode<T> {
        let new_node = LinkedListNode::new(value, Some(self));
        return new_node;
    }

    pub fn get_next(&self) -> *const LinkedListNode<T> {
        return self.next.unwrap();
    }

    pub fn has_next(&self) -> bool {
        return self.next.is_some();
    }
}

unsafe impl<T> Send for LinkedList<T> {}
unsafe impl<T> Sync for LinkedList<T> {}
