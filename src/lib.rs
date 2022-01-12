//! Double Cirlce Linked List
//! 
//! `insert_front` will insert an element at the front of the list
//! `insert-back` will insert an element at the end of the list
//! `delete` removes an element from the list
//! 
//! This is not a production level implementation and does not work
//! with different allocators
use std::marker::PhantomData;
use std::alloc::{self, Layout};
use std::ptr;
use std::fmt;

struct Node<T> {
    data : T,
    next_ptr : *mut Node<T>,
    prev_ptr : *mut Node<T>
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        println!("node has been dropped") 
    }
}

pub struct DoubleList<T> 
    where T : fmt::Display
{
    head : *mut Node<T>,
}

impl<T> DoubleList<T> 
    where T : fmt::Display
{
    pub fn new() -> Self {
        DoubleList {
            head : ptr::null_mut(),
        }
    }

    /// inserts an element at the start of the list
    pub fn insert_front(&mut self, new_data : T) {
        // create a node with the given element
        let node = unsafe {
            DoubleList::create_node(new_data)
        };

        println!("New node: {:p}", node);

        // in case the head is dangling, set the new node as the head
        // make it circular on itself
        if self.head == ptr::null_mut() {
            self.head = node;

            unsafe {
                // we need to change the next / prev to point to the head it self
                (*node).next_ptr = node;
                (*node).prev_ptr = node;

                println!("head: {:p}, next: {:p}, prev: {:p}", self.head, (*self.head).next_ptr, (*self.head).prev_ptr);
            }
        }
        else {
            // make the new node the head of the list, with:
            // new->prev = head->prev
            // head->prev->next = new (tail's next will be the new node)
            // head->prev = new
            // new->next = head
            unsafe {
                let tail = (*self.head).prev_ptr;

                // new node's prev will be head's prev
                (*node).prev_ptr = (*self.head).prev_ptr;

                // tails's next is going to be this new node
                (*tail).next_ptr = node;

                // head's prev will be new node
                (*self.head).prev_ptr = node;
                
                // new node's next will be head
                (*node).next_ptr = self.head;

                // head will change
                self.head = node;
                
                println!("head: {:p}, next: {:p}, prev: {:p}", self.head, (*self.head).next_ptr, (*self.head).prev_ptr);
            }
        }
    }

    /// returns an iterator on the elements
    pub fn iter(&self) -> ListIterator<'_, T> {
        ListIterator::new(self.head)
    }

    // creates a new node with the given element
    unsafe fn create_node(value : T) -> *mut Node<T> {
        let layout = Layout::for_value(&value);

        // allocate memory that can hold the layout of T (T's size and Alignment)
        let ptr = alloc::alloc(layout);
        let node_ptr = ptr as *mut Node<T>;

        (*node_ptr).data = value;
        (*node_ptr).next_ptr = ptr::null_mut();
        (*node_ptr).prev_ptr = ptr::null_mut();

        // // create a NonNull node from the raw pointer
        // let node = match NonNull::new(node_ptr) {
        //     Some(ptr) => ptr,
        //     None => alloc::handle_alloc_error(layout)   // this is what Box does in its implementation
        // };

        node_ptr
    }

    pub fn print(&self, len : i32) {
        let mut current : *const Node<T> = self.head;

        for i in 0..len {
            let value = unsafe {
                &(*current).data
            };

            unsafe {
                println!("value: {}, node: {:p}, next: {:p},", value, current, (*current).next_ptr);
                current = (*current).next_ptr;
            }
        }
    }
}

/// Drop implementation to dealloc nodes that were created using
/// alloc::alloc during the list's life
impl<T> Drop for DoubleList<T> 
    where T : fmt::Display
{
    fn drop(&mut self) { 
        // an empty list?
        if self.head == ptr::null_mut() {
            return;
        }

        // keep deleting each node till we reach the head node back again
        // even though we would have deleted the head by the time we will
        // reach to it but the pointer address is what we would be checking
        let end_ptr = self.head;
        let mut current = self.head;
        let layout = Layout::new::<T>();
        
        unsafe {
            loop {
                // remember the next before deleting the current node
                let next = (*current).next_ptr;
                // dealloc the memory for the current node
                alloc::dealloc(current.cast(), layout);

                // stop the loop in case we reach the end (next of a node == head pointer address)
                if next == end_ptr {
                    break;
                }
                else {
                    current = next;
                }
            }
        }
    }
}

/// Returns an iterator on the list
pub struct ListIterator<'a, T> {
    head_ptr : *const Node<T>,        // for reference to know when we have reached the end of the list
    iter_ptr : *const Node<T>,        // the current node that is being visited
    _phantom_data : PhantomData<&'a T>
}

impl<'a, T> ListIterator<'a, T> {
    // creates a new iterator. For emtpy list the current node
    // is set to dangling
    fn new(head_ptr : *const Node<T>) -> Self{
        ListIterator {
            head_ptr : head_ptr,
            iter_ptr : head_ptr,
            _phantom_data : PhantomData{}
        }
    }
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> { 
        // stop in case the iterator is dangling, this would happen
        // in case of empty list AND also when we reach the end of the list
        if self.iter_ptr == ptr::null() {
            return None
        }
        
        let current = self.iter_ptr;

        // get the value to return to the caller
        let value = unsafe {
            &(*current).data
        };

        // set the next node based on the next link of the node being
        // visited at the moment
        unsafe {
            let next : *const Node<T> = (*current).next_ptr;
            // is next pointing to the location of the head?
            if next == self.head_ptr {
                self.iter_ptr = ptr::null();
            }
            else {
                self.iter_ptr = next;
            }
        };

        Some(value)
    }
}


#[test]
fn list_insert() {
    let sample = [1,4,43];

    let mut list : DoubleList<i32> = DoubleList::new();
    for s in sample {
        list.insert_front(s);
    }

    list.print(3);

    let mut i = sample.len() - 1;
    for x in list.iter() {
        assert_eq!(*x, sample[i]);
        if i > 0 {
            i -= 1;
        }
    }
}