mod extension;
pub mod iterate;
pub use extension::*;
pub use iterate::iterate;

#[cfg(test)]
mod test {
    use parcom_core::Parser;

    use crate::standard::*;
    use crate::ParseResult;

    #[test]
    fn test() {
        fn numeric(input: &str) -> ParseResult<&str, &str, ()> {
            match input.char_indices().take_while(|t| t.1.is_numeric()).last() {
                Some((off, c)) => {
                    let idx = off + c.len_utf8();
                    Ok((&input[..idx], &input[idx..]))
                }
                None => Err(((), input)),
            }
        }

        let ten = numeric
            .iterate(|iter| -> Result<_, ()> {
                Ok(iter.take(10).map_while(|e| e.ok()).collect::<Vec<_>>())
            })
            .parse("1000a");

        println!("{:?}", ten)
    }
}
