use std::mem::MaybeUninit;

pub trait BufferWriter {
    type Segment: ?Sized;
    type Item;
    type Result;
    type Error;

    /// return capacity of the underlying buffer.
    fn capacity(&self) -> usize;
    /// return current count of written items.
    fn len(&self) -> usize;
    /// return pointer to the underlying buffer.
    fn as_ptr(&self) -> *const Self::Item;
    /// return pointer to the underlying buffer.
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    unsafe fn set_len(&mut self, new_len: usize);

    fn advance(self) -> Self::Result;
    fn cancel(self, err: Self::Error) -> Self::Result;

    fn as_slice(&self) -> &[Self::Item] {
        let ptr = self.as_ptr();
        let len = self.len();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        let ptr = self.as_mut_ptr();
        let len = self.len();
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }

    fn spare_capacity(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        let written = self.len();
        let len = self.capacity() - written;
        unsafe {
            let ptr = self.as_mut_ptr().add(written);
            std::slice::from_raw_parts_mut(ptr as *mut MaybeUninit<Self::Item>, len)
        }
    }

    fn push<T>(&mut self, item: T) -> Result<(), T>
    where
        T: WriteTo<Self::Segment, Self::Item>,
    {
        item.write_to(self)
    }

    fn push_item(&mut self, item: Self::Item) -> Result<(), Self::Item> {
        let written = self.len();
        if written == self.capacity() {
            return Err(item);
        }

        let ptr = self.as_mut_ptr();
        unsafe {
            ptr.add(written).write(item);
            self.set_len(written + 1);
        }

        Ok(())
    }

    fn shrink_to(&mut self, new_len: usize) {
        let len = self.len();
        assert!(new_len <= len);

        if new_len == len {
            return;
        }

        let ptr = self.as_mut_ptr();

        unsafe {
            let slice = std::ptr::slice_from_raw_parts_mut(ptr.add(new_len), len - new_len);
            std::ptr::drop_in_place(slice);
            self.set_len(new_len);
        }
    }
}

pub trait WriteTo<Segment: ?Sized, Item>: Sized {
    fn write_to<W>(self, writer: &mut W) -> Result<(), Self>
    where
        W: ?Sized + BufferWriter<Segment = Segment, Item = Item>;
}
