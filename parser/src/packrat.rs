mod boundcache;
mod cached;
mod smart_pointer;
use std::{cell::RefCell, collections::BTreeMap, marker::PhantomData};

use crate::{internal::Sealed, ParseStream, Parser, Stream};

use self::cached::Cached;
// この形式だと，キャッシュの削除が出来ない．
// バックトラックが発生しない部分のキャッシュを削除したい．
// しかしグローバルにキャッシュサーバを用意する場合は，適切なキャッシュの保存が難しい．
// advanceを監視して，キャッシュの放棄を適切に行う方法を考える．

// Stream側にサーバを持たせたり，advanceへのhookを付けたりすると，advanceの度にチェックが挟まり性能が悪化する．
// Segment内にキャッシュを保持させ， 利用する側は弱参照の二分木を作るとよいか？
// weak参照の一般化をする必要がある．Rcに依存した型は避けたい．

pub trait BindStream: ParseStream {
    type Weak<T>: smart_pointer::WeakRef<T>;
    fn bind<T>(self, index: usize, value: T) -> (Self::Weak<T>, Self);
}

pub trait PackratExtension<S: ParseStream>: Parser<S> + Sealed {
    /// The input must always be in the same stream.
    /// If a different stream is provided as input, it will produce incorrect results.
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

#[cfg(test)]
mod test {
    use crate::{standard::StandardExtension, Parser};

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
