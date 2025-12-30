use std::{rc::Rc, sync::Arc};

pub trait BufferStrategy {
    fn calc_capacity(&self, min_capacity: usize) -> usize;
}

impl<B: BufferStrategy> BufferStrategy for Arc<B> {
    fn calc_capacity(&self, min_capacity: usize) -> usize {
        B::calc_capacity(&self, min_capacity)
    }
}

impl<B: BufferStrategy> BufferStrategy for Rc<B> {
    fn calc_capacity(&self, min_capacity: usize) -> usize {
        B::calc_capacity(&self, min_capacity)
    }
}

impl<B: BufferStrategy> BufferStrategy for &B {
    fn calc_capacity(&self, min_capacity: usize) -> usize {
        B::calc_capacity(&self, min_capacity)
    }
}
