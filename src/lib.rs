//! Double Cirlce Linked List
//! 
//! Provides a safe interface to an unsafe implemented double linked list. This
//! is not a production level implementation and does not work with different 
//! allocators. It has been written just to see how unsafe code will be implemented.
//! 
//! `insert_front` will insert an element at the front of the list
//! `insert-back` will insert an element at the end of the list
//! `delete` removes an element from the list
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::alloc::{self, Layout};
use std::ptr;
use std::fmt;

#[derive(Debug)]
pub enum ListError {
    NodeNotFound
}

impl std::error::Error for ListError {}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ListError::NodeNotFound => {
                write!(f, "Value could not be found in the list")
            }
        }
    }
}

/// Each node of the linked list has a generic T element
/// a next and a prev pointer
struct Node<T> {
    data : T,
    next_ptr : *mut Node<T>,
    prev_ptr : *mut Node<T>
}

/// A double linked list has a head node. This implementation
/// does not use an empty head node. The head pointer is set to
/// ptr::null for an empty list
pub struct DoubleList<T> 
    where T : fmt::Display + PartialEq
{
    head : *mut Node<T>,
}

impl<T> Default for DoubleList<T> 
    where T : fmt::Display + PartialEq
{
    fn default() -> Self {
        Self::new()
    }
}
       
impl<T> DoubleList<T> 
    where T : fmt::Display + PartialEq
{
    pub fn new() -> Self {
        DoubleList {
            head : ptr::null_mut(),
        }
    }

    /// is the list empty?
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// inserts an element at the start of the list
    pub fn insert_front(&mut self, new_data : T) {
        // create a node with the given element
        let node = unsafe {
            DoubleList::create_node(new_data)
        };

        // in case the head is dangling, set the new node as the head
        // make it circular on itself
        if self.head.is_null() {
            self.head = node;

            unsafe {
                // we need to change the next / prev to point to the head it self
                (*node).next_ptr = node;
                (*node).prev_ptr = node;
            }
        }
        else {
            // make the new node the head of the list, with:
            // new->prev = head->prev (tail)
            // tail->next = new node
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
            }
        }
    }

    /// removes the nth node from the list
    /// True in case the item was removed otherwise False
    pub fn remove(&mut self, _index : i32) -> bool {
        todo!()
    }

    /// inserts an element at the back of the list
    /// inserts an element at the start of the list
    pub fn insert_back(&mut self, new_data : T) {
        // create a node with the given element
        let node = unsafe {
            DoubleList::create_node(new_data)
        };

        // in case the head is dangling, set the new node as the head
        // make it circular on itself
        if self.head.is_null() {
            self.head = node;

            unsafe {
                // we need to change the next / prev to point to the head it self
                (*node).next_ptr = node;
                (*node).prev_ptr = node;
            }
        }
        else {
            // make the new node the tail of the list, with:
            // new->prev = tail
            // tail->next = new node
            // head->prev = new node
            // new->next = head
            unsafe {
                let tail = (*self.head).prev_ptr;

                // new node's prev will be the current tail
                (*node).prev_ptr = tail;

                // tail's next is going to be this new node
                (*tail).next_ptr = node;

                // head's prev will be new node since this is the new tail
                (*self.head).prev_ptr = node;
                
                // new node's next will be head
                (*node).next_ptr = self.head;
            }
        }
    }


    /// Deletes an element with value fromt the list. It will remove 
    /// the first node that has the element. 
    /// returns false in case the element was not found
    pub fn delete(&mut self, element : &T) -> Result<(), ListError> {
        // can't delete in case the value does not exist
        if self.head.is_null() {
            return Err(ListError::NodeNotFound);
        }

        let mut cur = self.head;
        loop {
            unsafe {
                let value = &(*cur).data;

                if *value == *element {
                    self.remove_node(cur);
                    return Ok(());
                }
                else {
                    cur = (*cur).next_ptr;
                    // have we cycled hte list?
                    if cur == self.head {
                        break;
                    }
                }
            }
        }

        Err(ListError::NodeNotFound)
    }

    /// returns an iterator on the elements
    pub fn iter(&self) -> ListIterator<'_, T> {
        ListIterator::new(self.head)
    }

    /// returns an iterator on the elements
    pub fn iter_rev(&self) -> RevListIterator<'_, T> {
        RevListIterator::new(self.head)
    }

    /// the node is moved into the parameter so that it cannot
    /// be used afterwards
    fn remove_node(&mut self, cur : *mut Node<T>) {
        unsafe {
            // is this the only node in the list? if yes 
            // no need to change any pointers
            if (*self.head).next_ptr == self.head {
                self.head = ptr::null_mut();
            }
            else {
                (*(*cur).prev_ptr).next_ptr = (*cur).next_ptr;
                (*(*cur).next_ptr).prev_ptr = (*cur).prev_ptr;

                // change head in case this is the head node
                if self.head == cur {
                    self.head = (*cur).next_ptr;
                }
            }

            // remove memory for the node
            let layout = Layout::new::<Node<T>>();
            alloc::dealloc(cur.cast(), layout);
        }
    }
    
    // creates a new node with the given element
    unsafe fn create_node(value : T) -> *mut Node<T> {
        // allocate memory that can hold the layout of T (T's size and Alignment)
        let layout = Layout::new::<Node<T>>();
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
}

/// Drop implementation to dealloc nodes that were created using
/// alloc::alloc during the list's life
impl<T> Drop for DoubleList<T> 
    where T : fmt::Display + PartialEq
{
    fn drop(&mut self) { 
        // an empty list?
        if self.head.is_null() {
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
            head_ptr,
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
        if self.iter_ptr.is_null() {
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

/// Returns a reverse iterator on the list
pub struct RevListIterator<'a, T> {
    tail_ptr : *const Node<T>,        // for reference to know when we have reached the end of the list
    iter_ptr : *const Node<T>,        // the current node that is being visited
    _phantom_data : PhantomData<&'a T>
}

/// A reverse iterator that uses the prev pointer to traverse
impl<'a, T> RevListIterator<'a, T> {
    // creates a new iterator. For emtpy list the current node
    // is set to dangling
    fn new(head_ptr : *const Node<T>) -> Self{
        // remember the tail pointer to find out when the list is back on the tail
        let tail_ptr = if head_ptr.is_null() {
                ptr::null()
            }
            else {
                unsafe {
                    (*head_ptr).prev_ptr
                }
            };

        RevListIterator {
            tail_ptr,
            iter_ptr : tail_ptr,
            _phantom_data : PhantomData{}
        }
    }
}

impl<'a, T> Iterator for RevListIterator<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> { 
        // stop in case the iterator is dangling, this would happen
        // in case of empty list AND also when we reach the end of the list
        if self.iter_ptr.is_null() {
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
            let prev : *const Node<T> = (*current).prev_ptr;
            // is next pointing to the location of the head?
            if prev == self.tail_ptr {
                self.iter_ptr = ptr::null();
            }
            else {
                self.iter_ptr = prev;
            }
        };

        Some(value)
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    #[test]
    fn test_insert_front() {
        let sample = vec![1,4,43, 9, 3, 56, 4];

        let mut list : crate::DoubleList<i32> = crate::DoubleList::new();
        for s in &sample {
            list.insert_front(*s);
        }

        test_forward(&sample, &list);
        test_reverse(&sample, &list);
    }

    #[test]
    fn test_insert_back() {
        let mut rng = rand::thread_rng();
        let mut sample : Vec<i32> = Vec::with_capacity(64);

        for _ in 0..64 {
            let x : i32 = rng.gen();
            sample.push(x);
        }

        let mut list : crate::DoubleList<i32> = crate::DoubleList::new();
        for s in &sample {
            list.insert_back(*s);
        }

        // the two functions test_forward and test_reverse were written
        // from the insert_front angle so we need to reverse the sample list
        sample.reverse();
        // test if they match
        test_forward(&sample, &list);
        test_reverse(&sample, &list);
    }

    fn test_forward<T>(sample : &Vec<T>, list : &crate::DoubleList<T>) 
        where T : std::fmt::Display + std::fmt::Debug + std::cmp::PartialEq
    {
        let mut i = sample.len() - 1;
        for x in list.iter() {
            assert_eq!(*x, sample[i]);
            if i > 0 {
                i -= 1;
            }
        }
    }

    fn test_reverse<T>(sample : &Vec<T>, list : &crate::DoubleList<T>) 
        where T : std::fmt::Display + std::fmt::Debug + std::cmp::PartialEq
    {
        let mut i = 0;
        for x in list.iter_rev() {
            assert_eq!(*x, sample[i]);
            i += 1;
        }
    }

    /// checks if returning a double list from a function works
    #[test]
    fn test_ret_from_fn() {
        let sample = vec![1,4,43, 4, 5, 7, 9, 10, 11];
        let list = add_to_list(&sample);

        test_forward(&sample, &list);
        test_reverse(&sample, &list);
    }

    fn add_to_list(sample : &Vec<i32>) -> crate::DoubleList<i32> {
        let mut list : crate::DoubleList<i32> = crate::DoubleList::new();
        for s in sample {
            list.insert_front(*s);
        }

        list
    }

    /// checks if returning a double list from a function works
    #[test]
    fn test_delete() {
        let mut sample = vec![1,4,43, 4, 5, 7, 9, 10, 11];
        let mut list = add_to_list(&sample);
        
        assert!(list.delete(&43).is_ok(), "delete could not find node");
        assert!(list.delete(&1043).is_err(), "delete did not return error");

        sample.remove(2);

        test_forward(&sample, &list);
        test_reverse(&sample, &list);
    }
}