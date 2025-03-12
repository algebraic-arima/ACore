use core::{fmt, ptr};


#[derive(Copy, Clone)]
pub struct LinkedList {
    pub head: *mut usize,
}

impl LinkedList {
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub unsafe fn push(&mut self, item: *mut usize) {
        *item = self.head as usize;
        self.head = item;
    }

    pub fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                let item = self.head;
                self.head = unsafe { *item as *mut usize };
                Some(item)
            }
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,
            linked_list: self,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            linked_list: self,
        }
    }
}

pub struct Iter<'a> {
    curr: *mut usize,
    linked_list: &'a LinkedList,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = self.curr;
            self.curr = unsafe { *self.curr } as *mut usize;
            Some(result)
        } else {
            None
        }
    }
}

pub struct LinkedListInner {
    prev: *mut usize,
    curr: *mut usize,
}

impl LinkedListInner {
    pub fn as_ptr(&self) -> *mut usize {
        self.curr
    }

    pub fn pop(self) -> *mut usize {
        unsafe {
            *self.prev = *self.curr;
        }
        self.curr
    }
}

pub struct IterMut<'a> {
    prev: *mut usize,
    curr: *mut usize,
    linked_list: &'a mut LinkedList,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = LinkedListInner;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = LinkedListInner {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe { *self.curr } as *mut usize;
            Some(result)
        } else {
            None
        }
    }
}