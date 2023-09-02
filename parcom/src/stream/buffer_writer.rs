use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

pub struct BufferWriter<T> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
}

impl<T> BufferWriter<T> {
    pub fn new(ptr: *mut T, capacity: usize) -> Self {
        Self {
            ptr,
            len: 0,
            capacity,
        }
    }

    pub fn from(buf: &mut [MaybeUninit<T>]) -> Self {
        Self::new(buf.as_mut_ptr().cast(), buf.len())
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    pub fn as_slice(&self) -> &[T] {
        let ptr = self.ptr;
        let len = self.len;

        unsafe { std::slice::from_raw_parts(ptr.cast(), len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let ptr = self.ptr;
        let len = self.len;

        unsafe { std::slice::from_raw_parts_mut(ptr.cast(), len) }
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.capacity == self.len {
            return Err(item);
        }

        unsafe { self.ptr.add(self.len).write(item) };
        self.len += 1;
        Ok(())
    }

    pub fn insert(&mut self, index: usize, item: T) -> Result<(), T> {
        let len = self.len;
        if len < index {
            panic!("index out of bounds.");
        }
        let overflowed;
        let len = unsafe {
            let ptr = self.ptr.add(index) as *mut T;
            let len = if len == self.capacity {
                overflowed = Err(ptr.add(len - 1).read());
                len - 1
            } else {
                overflowed = Ok(());
                len
            };

            if index < len {
                std::ptr::copy(ptr, ptr.add(1), len - index);
            }

            std::ptr::write(ptr, item);
            len
        };
        self.len = len + 1;
        overflowed
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let ptr = self.ptr;
        self.len -= 1;
        let item = unsafe { ptr.add(self.len).read() };
        Some(item)
    }

    pub fn remove(&mut self, index: usize) -> T {
        if self.len <= index {
            panic!("index out of bounds.");
        }
        unsafe {
            let ptr = self.ptr.add(index);
            let ret = ptr.read();
            self.len -= 1;
            std::ptr::copy(ptr.add(1), ptr, self.len - index);
            ret
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        if self.len <= index {
            panic!("index out of bounds.");
        }

        let ptr = self.ptr;
        unsafe {
            let dst = ptr.add(index);
            let ret = dst.read();
            self.len -= 1;
            dst.write(ptr.add(self.len).read());
            ret
        }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr.add(self.len).cast(), self.capacity - self.len)
        }
    }

    pub fn extend_from_iter<I: Iterator<Item = T>>(&mut self, mut iter: I) -> I {
        let ptr = self.ptr;
        let mut len = self.len;
        loop {
            if self.capacity <= len {
                break;
            }

            let Some(item) = iter.next() else {
                break;
            };

            unsafe { ptr.add(len).write(item) };

            len += 1;
        }

        self.len = len;
        iter
    }
}

impl<T> Deref for BufferWriter<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for BufferWriter<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
