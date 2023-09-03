mod into_parser;
use std::marker::PhantomData;

use crate::{internal::Sealed, Parse};

pub trait ParseExtension<T>: Parse<T> + Sealed {
    fn into_parser() -> into_parser::IntoParser<T, Self> {
        into_parser::IntoParser(PhantomData)
    }
}
