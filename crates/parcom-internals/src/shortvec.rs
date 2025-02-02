use std::{
    alloc::{self, Layout},
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct ShortVec<T, const N: usize> {
    /// capacity of vec.
    ///
    /// invariant: `cap >= N` & `cap <= isize::MAX`
    cap: usize,
    len: usize,
    /// elements of vec. if `cap > N`, data is heap, otherwise data is stack
    data: ShortVecData<T, N>,
}

union ShortVecData<T, const N: usize> {
    stack: ManuallyDrop<[MaybeUninit<T>; N]>,
    heap: ManuallyDrop<NonNull<T>>,
}

impl<T, const N: usize> ShortVec<T, N> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(cap: usize) -> Self {
        if cap > N {
            let layout = Layout::array::<T>(cap).unwrap();
            let ptr = unsafe { alloc::alloc(layout) as *mut T };
            let ptr = match NonNull::new(ptr) {
                Some(v) => v,
                None => alloc::handle_alloc_error(layout),
            };

            let data = ShortVecData {
                heap: ManuallyDrop::new(ptr),
            };
            Self { cap, len: 0, data }
        } else {
            let data = ShortVecData {
                stack: ManuallyDrop::new(std::array::from_fn(|_| MaybeUninit::uninit())),
            };

            Self {
                cap: N,
                len: 0,
                data,
            }
        }
    }

    pub fn as_ptr(&self) -> *const T {
        if self.is_heap() {
            unsafe { self.as_ptr_heap_unchecked() }
        } else {
            unsafe { self.as_ptr_stack_unchecked() }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        if self.is_heap() {
            unsafe { NonNull::slice_from_raw_parts(*self.data.heap, self.len).as_ref() }
        } else {
            unsafe { std::mem::transmute(&self.data.stack.deref()[..self.len]) }
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        if self.is_heap() {
            unsafe { NonNull::slice_from_raw_parts(*self.data.heap, self.len).as_mut() }
        } else {
            unsafe { std::mem::transmute(&mut self.data.stack.deref_mut()[..self.len]) }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, item: T) {
        let len = self.len;
        self.reserve(len + 1);
        let ptr = self.as_ptr() as *mut T;
        unsafe {
            ptr.add(len).write(item);
        }
        self.len = len + 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let ptr = self.as_ptr() as *mut T;
        self.len -= 1;
        let value = unsafe { ptr.add(self.len).read() };
        Some(value)
    }

    pub fn reserve(&mut self, cap: usize) {
        if self.cap >= cap {
            return;
        }
        let cap = usize::max(self.cap * 2, cap);
        Self::reserve_exact(self, cap)
    }

    pub fn reserve_exact(&mut self, cap: usize) {
        assert!(
            cap <= isize::MAX as usize,
            "`cap` must not be greater than isize::MAX"
        );

        if self.cap >= cap {
            return;
        }
        // `cap`の不変条件によって、`self.cap >= N`をかならず満たすため、ここではヒープアロケーションが必ず行われる。
        let layout = Layout::array::<T>(cap).unwrap();
        if self.is_heap() {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let ptr = unsafe {
                let ptr = self.data.heap.as_ptr() as *mut u8;
                alloc::realloc(ptr, old_layout, layout.size()) as *mut T
            };

            let ptr = match NonNull::new(ptr) {
                Some(v) => ManuallyDrop::new(v),
                None => alloc::handle_alloc_error(layout),
            };

            self.cap = cap;
            self.data.heap = ptr;
        } else {
            let buf = unsafe {
                let ptr = alloc::alloc(layout) as *mut T;
                match NonNull::new(ptr) {
                    Some(v) => v,
                    None => alloc::handle_alloc_error(layout),
                }
            };

            unsafe {
                let src = self.as_ptr_stack_unchecked();
                std::ptr::copy_nonoverlapping(src, buf.as_ptr(), self.len);
            };

            self.cap = cap;
            self.data.heap = ManuallyDrop::new(buf);
        }
    }
}

impl<T, const N: usize> ShortVec<T, N> {
    fn is_heap(&self) -> bool {
        self.cap > N
    }

    unsafe fn as_ptr_stack_unchecked(&self) -> *const T {
        unsafe { self.data.stack.as_ptr() as *const T }
    }

    unsafe fn as_ptr_heap_unchecked(&self) -> *const T {
        unsafe { self.data.heap.as_ptr() }
    }
}

impl<T, const N: usize> Drop for ShortVec<T, N> {
    fn drop(&mut self) {
        let ptr = self.as_ptr() as *mut T;
        let slice = std::ptr::slice_from_raw_parts_mut(ptr, self.len);
        unsafe {
            std::ptr::drop_in_place(slice);
        }

        if self.is_heap() {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    }
}

impl<T, const N: usize> Default for ShortVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockalloc::Mockalloc;

    #[global_allocator]
    static ALLOCATOR: Mockalloc<std::alloc::System> = Mockalloc(std::alloc::System);

    #[test]
    fn never_allocation() {
        const N: usize = 4;

        let info = mockalloc::record_allocs(|| {
            drop(ShortVec::<Box<()>, N>::new());
        });

        assert_eq!(info.num_allocs(), 0);
        assert_eq!(info.num_frees(), 0);
    }

    #[test]
    fn should_spill_to_heap_on_push_to_size_zero_vec() {
        const BUF_LEN: usize = 0;
        const ITEM_COUNT: usize = 1;

        let info = mockalloc::record_allocs(|| {
            let mut vec = ShortVec::<_, BUF_LEN>::new();
            for i in 0..ITEM_COUNT {
                vec.push(Box::new(i));
            }

            for _ in 0..(ITEM_COUNT / 2) {
                vec.pop();
            }
        });

        assert_eq!(info.num_allocs(), (ITEM_COUNT + 1) as u64);
        assert_eq!(info.num_frees(), (ITEM_COUNT + 1) as u64);
    }

    #[test]
    fn should_not_spill_with_enough_capacity() {
        const N: usize = 4;

        let info = mockalloc::record_allocs(|| {
            let mut vec = ShortVec::<_, N>::new();
            for i in 0..N {
                vec.push(Box::new(i));
            }

            for _ in 0..(N / 2) {
                vec.pop();
            }
        });

        assert_eq!(info.num_allocs(), N as u64);
        assert_eq!(info.num_frees(), N as u64);
    }

    #[test]
    fn should_spill_when_stack_buffer_overflowed() {
        const BUF_LEN: usize = 4;
        const ITEM_COUNT: usize = BUF_LEN + 1;

        let info = mockalloc::record_allocs(|| {
            let mut vec = ShortVec::<_, BUF_LEN>::new();
            for i in 0..ITEM_COUNT {
                vec.push(Box::new(i));
            }

            for _ in 0..(ITEM_COUNT / 2) {
                vec.pop();
            }
        });

        assert_eq!(info.num_allocs(), (ITEM_COUNT + 1) as u64);
        assert_eq!(info.num_frees(), (ITEM_COUNT + 1) as u64);
    }
}
