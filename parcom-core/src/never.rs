pub enum Never {}

pub trait ShouldNever {
    fn never<T>(&self) -> T {
        unreachable!("ShouldNever::never should be never called.")
    }
}

impl ShouldNever for Never {}
