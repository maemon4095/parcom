use std::ops::Bound;

pub trait Sealed {}
impl<T> Sealed for T {}

pub fn just_on_boundary(item: usize, bound: Bound<&usize>) -> bool {
    match bound {
        Bound::Included(e) => item == *e,
        Bound::Excluded(e) => item + 1 == *e,
        Bound::Unbounded => false,
    }
}
