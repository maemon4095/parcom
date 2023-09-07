pub mod binary_expr;
pub mod parse;
pub mod parser;

use std::ops::Bound;

pub use parser::ParserExtension;

fn just_on_boundary(item: usize, bound: Bound<&usize>) -> bool {
    match bound {
        Bound::Included(e) => item == *e,
        Bound::Excluded(e) => item + 1 == *e,
        Bound::Unbounded => false,
    }
}

#[derive(Debug, Clone)]
pub enum Either<T0, T1> {
    First(T0),
    Last(T1),
}
