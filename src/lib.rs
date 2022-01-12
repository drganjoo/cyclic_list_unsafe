//! Double Cirlce Linked List
//! 
//! `insert_front` will insert an element at the front of the list
//! `insert-back` will insert an element at the end of the list
//! `delete` removes an element from the list
//! 
//! This is not a production level implementation and does not work
//! with different allocators
use std::alloc::{self, Layout};
use std::ptr::NonNull;
use std::fmt;

struct Node<T> {
    data : T,
    next_ptr : NonNull<Node<T>>,
    prev_ptr : NonNull<Node<T>>
}

pub struct DoubleList<T> 
    where T : fmt::Display
{
    head : NonNull<Node<T>>,
}

impl<T> DoubleList<T> 
    where T : fmt::Display
{
    pub fn new() -> Self {
        DoubleList {
            head : NonNull::dangling(),
        }
    }

    /// inserts an element at the start of the list
    pub fn insert_front(&mut self, new_data : T) {
        // create a node with the given element
        let node = unsafe {
            DoubleList::create_node(new_data)
        };
        
        // in case the head is dangling, set the new node as the head
        // make it circular on itself
        if self.head == NonNull::dangling() {
            self.head = node;

            unsafe {
                // we need to change the next / prev to point to the head it self
                let mut head_ptr = self.head.as_ptr();
                (*head_ptr).next_ptr = NonNull::new_unchecked(head_ptr);
                (*head_ptr).prev_ptr = NonNull::new_unchecked(head_ptr);
            }
        }
        else {
            // make the new node the head of the list, with:
            // new->prev = head->prev
            // head->prev->next = new (tail's next will be the new node)
            // head->prev = new
            // new->next = head
            unsafe {
                let node_ptr = node.as_ptr();
                let head_ptr = self.head.as_ptr();

                // new node's prev will be head's prev
                (*node_ptr).prev_ptr = NonNull::new_unchecked((*head_ptr).prev_ptr.as_ptr());
                
                // tails's next is going to be this new node
                let tail = (*head_ptr).prev_ptr;
                (*tail.as_ptr()).next_ptr = NonNull::new_unchecked(node_ptr);

                // head's prev will be new node
                (*head_ptr).prev_ptr = NonNull::new_unchecked(node_ptr);
                
                // new node's next will be head
                (*node_ptr).next_ptr = NonNull::new_unchecked(head_ptr);

                // head will change
                self.head = node;
            }
        }
    }

    /// returns an iterator on the elements
    pub fn iter(&self) -> ListIterator<'_, T> {
        ListIterator::new(&self.head)
    }

    // creates a new node with the given element
    unsafe fn create_node(value : T) -> NonNull<Node<T>> {
        let layout = Layout::for_value(&value);

        // allocate memory that can hold the layout of T (T's size and Alignment)
        let ptr = alloc::alloc(layout);
        let node_ptr = ptr as *mut Node<T>;

        (*node_ptr).data = value;
        (*node_ptr).next_ptr = NonNull::dangling();
        (*node_ptr).prev_ptr = NonNull::dangling();

        // create a NonNull node from the raw pointer
        let node = match NonNull::new(node_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout)   // this is what Box does in its implementation
        };

        node
    }

    pub fn print(&self, len : i32) {
        let mut head = &self.head;
        for i in 0..len {
            let ptr = head.as_ptr();
            let value = unsafe {
                &(*ptr).data
            };
            println!("{}", value);

            unsafe {
                head = &(*head.as_ptr()).next_ptr;
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
        if self.head == NonNull::dangling() {
            return;
        }

        // keep deleting each node till we reach the head node back again
        // even though we would have deleted the head by the time we will
        // reach to it but the pointer address is what we would be checking
        let end_ptr = self.head.as_ptr();
        let mut current = self.head.as_ptr();
        let layout = Layout::new::<T>();
        
        unsafe {
            loop {
                // remember the next before deleting the current node
                let next = ((*current).next_ptr).as_ptr();
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
    head_ptr : &'a NonNull<Node<T>>,        // for reference to know when we have reached the end of the list
    iter_ptr : NonNull<Node<T>>,            // the current node that is being visited
}

impl<'a, T> ListIterator<'a, T> {
    // creates a new iterator. For emtpy list the current node
    // is set to dangling
    fn new(head_ptr : &'a NonNull<Node<T>>) -> Self{
        ListIterator {
            head_ptr : head_ptr,
            iter_ptr : unsafe {
                // in case the list is empty set the current node being iterated
                // to dangling
                if *head_ptr == NonNull::dangling() {
                    NonNull::dangling()
                }
                else {
                    // use the head as the first element
                    NonNull::new_unchecked(head_ptr.as_ptr())
                }
            }
        }
    }
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> { 
        // stop in case the iterator is dangling, this would happen
        // in case of empty list AND also when we reach the end of the list
        if self.iter_ptr == NonNull::dangling() {
            return None
        }
        
        let current : NonNull<Node<T>> = self.iter_ptr;
        let node_ptr = current.as_ptr();

        // get the value to return to the caller
        let value = unsafe {
            &(*node_ptr).data
        };

        // set the next node based on the next link of the node being
        // visited at the moment
        unsafe {
            let next = (*node_ptr).next_ptr;
            // is next pointing to the location of the head?
            if next.as_ptr() == (*self.head_ptr).as_ptr() {
                self.iter_ptr = NonNull::dangling();
            }
            else {
                self.iter_ptr = NonNull::new_unchecked(next.as_ptr());
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