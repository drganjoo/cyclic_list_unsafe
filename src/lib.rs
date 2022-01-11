use std::alloc::{self, Layout};
use std::ptr::NonNull;
use std::marker::PhantomData;

struct Node<T> {
    data : T,
    next_ptr : NonNull<Node<T>>,
    prev_ptr : NonNull<Node<T>>
}

struct DoubleList<T> {
    head : NonNull<Node<T>>,
    _null_marker : PhantomData<T>,
}

impl<T> DoubleList<T> {
    pub fn new() -> Self {
        DoubleList {
            head : NonNull::dangling(),
            _null_marker : PhantomData
        }
    }

    pub fn insert_front(&mut self, new_data : T) {
        // create a new node
        let layout = Layout::for_value(&new_data);
        let node_ptr = unsafe {
            alloc::alloc(layout)
        };

        let mut node = node_ptr as *mut Node<T>;
        
        if self.head == NonNull::dangling() {
            unsafe {
                (*node).data = new_data;

                let node_non_null = match NonNull::new(node_ptr as *mut Node<T>) {
                    Some(ptr) => ptr,
                    None => alloc::handle_alloc_error(layout)
                };

                (*node).next_ptr = node_non_null;
                (*node).prev_ptr = node_non_null;
                self.head = node_non_null;
            }
        }
        else {
            unsafe {
                (*node).data = new_data;
                (*node).next_ptr = match NonNull::new(self.head.as_ptr() as *mut Node<T>) {
                    Some(ptr) => ptr,
                    None => alloc::handle_alloc_error(layout)
                };
                (*node).prev_ptr = (*self.head).prev_ptr;
                
                let node_non_null = match NonNull::new(node_ptr as *mut Node<T>) {
                    Some(ptr) => ptr,
                    None => alloc::handle_alloc_error(layout)
                };

                (*node).prev_ptr = node_non_null;
                self.head = node_non_null;
            }
        }
    }   
}
