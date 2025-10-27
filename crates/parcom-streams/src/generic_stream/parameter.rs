#[derive(Debug)]
pub struct GenericStreamParameter {
    min_capacity: usize, // セグメントバッファの最小容量
}

impl Default for GenericStreamParameter {
    fn default() -> Self {
        Self { min_capacity: 16 }
    }
}

impl GenericStreamParameter {
    pub fn with_min_capacity(mut self, min_capacity: usize) -> Self {
        self.min_capacity = min_capacity;
        self
    }

    pub(super) fn calc_new_segment_capacity(&self, min_size: usize) -> usize {
        usize::max(min_size, self.min_capacity)
    }
}
