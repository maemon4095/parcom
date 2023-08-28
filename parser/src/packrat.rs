use std::{cell::RefCell, collections::BTreeMap, marker::PhantomData};

use crate::{internal::Sealed, Location, ParseResult, ParseStream, Parser};
// この形式だと，キャッシュの削除が出来ない．
// バックトラックが発生しない部分のキャッシュを削除したい．
// しかしグローバルにキャッシュサーバを用意する場合は，適切なキャッシュの保存が難しい．
// advanceを監視して，キャッシュの放棄を適切に行う方法を考える．
pub trait PackratExtension<S: ParseStream>: Parser<S> + Sealed {
    fn cached(self) -> Cached<S, Self>
    where
        Self: Sized,
        Self::Error: Clone,
        Self::Output: Clone,
    {
        Cached {
            parser: self,
            server: RefCell::new(BTreeMap::new()),
            marker: PhantomData,
        }
    }
}

impl<S: ParseStream, P: Parser<S> + Sealed> PackratExtension<S> for P {}

pub struct Cached<S: ParseStream, P: Parser<S>>
where
    P::Error: Clone,
    P::Output: Clone,
{
    parser: P,
    server: RefCell<BTreeMap<S::Location, Result<(P::Output, usize), P::Error>>>,
    marker: PhantomData<S>,
}

impl<S: ParseStream, P: Parser<S>> Parser<S> for Cached<S, P>
where
    P::Error: Clone,
    P::Output: Clone,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self> {
        let location = input.location(0);
        match self.server.borrow().get(&location) {
            Some(result) => {
                return match result {
                    Ok((v, c)) => Ok((v.clone(), input.advance(*c))),
                    Err(e) => Err((e.clone(), input)),
                }
            }
            None => (),
        }

        match self.parser.parse(input) {
            Ok((v, r)) => {
                let tail = r.location(0);
                let distance = tail.distance(&location);
                self.server
                    .borrow_mut()
                    .insert(location, Ok((v.clone(), distance)));
                Ok((v, r))
            }
            Err((e, r)) => {
                self.server.borrow_mut().insert(location, Err(e.clone()));
                Err((e, r))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{standard_extension::StandardExtension, Parser};

    use super::PackratExtension;

    fn a() {
        let atom = crate::foreign::parser::str::atom("aaa");
        let atom1 = crate::foreign::parser::str::atom("bbb");
        let cached = atom.cached();

        let joined = cached.as_ref().join(atom1);

        let source = crate::foreign::stream::StrStream::new("aaabbb");
        joined.parse(source);
    }
}
