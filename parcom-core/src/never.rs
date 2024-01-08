#[derive(Debug, Clone, Copy)]
pub enum Never {}

pub trait ShouldNever {
    fn never<T>(&self) -> T {
        unreachable!("ShouldNever::never should never be called.")
    }
}

impl ShouldNever for Never {}
