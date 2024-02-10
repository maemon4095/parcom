pub unsafe trait ShouldNever {}

#[derive(Debug, Clone, Copy)]
pub enum Never {}
unsafe impl ShouldNever for Never {}

unsafe impl<T: ShouldNever, E: ShouldNever> ShouldNever for Result<T, E> {}
unsafe impl<T: ShouldNever, const N: usize> ShouldNever for [T; N] {}

macro_rules! tuple_impl {
    ($t:ident, $u:ident) => {
        tuple_impl!(@impl $t, $u);
    };

    ( $u:ident, $($t:ident),+ ) => {
        tuple_impl!(@impl $u, $($t),*);
        tuple_impl!($($t),*);
    };

    (@impl $($t:ident),+) => {
        unsafe impl<$($t: ShouldNever),*> ShouldNever for ($($t),*) {}
    };
}

tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

pub trait ShouldNeverExtension: ShouldNever {
    fn never<T>(&self) -> T {
        unreachable!(
            "<{} as ShouldNever>::never should never be called.",
            std::any::type_name::<Self>()
        )
    }
}

impl<T: ShouldNever> ShouldNeverExtension for T {}
