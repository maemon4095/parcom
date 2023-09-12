pub enum Never {}

pub trait ShouldNever {
    fn never(&self) -> Never {
        panic!("this function should be never called.")
    }
}

impl ShouldNever for Never {}
