pub unsafe trait ShouldNever {}

#[derive(Debug, Clone, Copy)]
pub enum Never {}
unsafe impl ShouldNever for Never {}

unsafe impl<T: ShouldNever, E: ShouldNever> ShouldNever for Result<T, E> {}
unsafe impl<T: ShouldNever, const N: usize> ShouldNever for [T; N] {}

pub trait ShouldNeverExtension: ShouldNever {
    fn never<T>(&self) -> T {
        unreachable!(
            "<{} as ShouldNever>::never should never be called.",
            std::any::type_name::<Self>()
        )
    }
}

impl<T: ShouldNever> ShouldNeverExtension for T {}

impl std::error::Error for Never {}
impl std::fmt::Display for Never {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}
