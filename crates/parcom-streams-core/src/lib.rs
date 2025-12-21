mod buffer_writer;

use std::future::Future;

pub use buffer_writer::BufferWriter;

pub trait StreamSource: Sized {
    type Item;
    type Error;
    type Next<'a, C>: Future<Output = C::Result>
    where
        Self: 'a,
        C: 'a + StreamControl<Item = Self::Item, Error = Self::Error>;

    fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
    where
        C: 'a + StreamControl<Item = Self::Item, Error = Self::Error>;
}

pub trait StreamControl {
    type Item;
    type Result;
    type Error;
    type Writer: BufferWriter<Item = Self::Item, Result = Self::Result, Error = Self::Error>;

    fn request_writer(self, min_capacity: usize) -> Self::Writer;
    fn cancel(self, err: Self::Error) -> Self::Result;
    fn finish(self) -> Self::Result;
}

pub trait StreamLoader {
    type Error;
    type Load<'a>: Future<Output = Result<Option<LoadInfo>, Self::Error>>
    where
        Self: 'a;

    fn set_min_buffer_size(&mut self, size: usize);

    /// データの読み込みを行う。データが追加された場合は`Ok(Some(LoadInfo))`、末尾に到達した場合は`Ok(None)`、エラーが発生した場合は`Err(err)`が返される。
    ///
    /// データ末尾に到達した場合、ストリームへのcommitも行われ、ローダー内部にデータは残らない。
    fn load(&mut self) -> Self::Load<'_>;
    fn force_commit(&mut self);
}

#[derive(Debug, Clone)]
pub struct LoadInfo {
    committed: usize,
    uncommitted: usize,
    buffer_capacity: usize,
}

impl LoadInfo {
    pub fn new(committed: usize, uncommitted: usize, buffer_capacity: usize) -> Self {
        Self {
            committed,
            uncommitted,
            buffer_capacity,
        }
    }

    /// ストリームに追加されたデータの量
    pub fn committed(&self) -> usize {
        self.committed
    }

    /// bufferに読み込まれているがストリームには追加されていないデータの量
    pub fn uncommitted(&self) -> usize {
        self.uncommitted
    }

    pub fn buffer_capacity(&self) -> usize {
        self.buffer_capacity
    }
}

// バッファロードのスケジューリングを行う。
pub trait StreamDriver<L: StreamLoader> {
    type Session: StreamDriverSession<Error = L::Error>;

    fn start(self, loader: L) -> Self::Session;
}

pub trait StreamDriverSession {
    type Error;
    // インスタンスが作成されてから初めにpollを呼ばれる前にデータが追加された場合、直ちに`Poll::Ready`を返す必要がある。
    type WaitForAppendData<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn is_terminated(&self) -> bool;
    fn notify_data_examined(&mut self, amount: usize);
    /// `wait_for_append_data`で`Err`が返された後、もう一度`wait_for_append_data`を呼び出してはならない。
    fn wait_for_append_data(&mut self, size_hint: usize) -> Self::WaitForAppendData<'_>;
}
