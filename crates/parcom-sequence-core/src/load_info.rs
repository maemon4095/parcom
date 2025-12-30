pub struct LoadInfo {
    commited: usize,
    uncommited: usize,
    buffer_capacity: usize,
    is_done: bool,
}

impl LoadInfo {
    pub fn new(commited: usize, uncommited: usize, buffer_capacity: usize) -> Self {
        Self {
            commited,
            uncommited,
            buffer_capacity,
            is_done: false,
        }
    }

    pub fn done(commited: usize) -> Self {
        Self {
            commited,
            uncommited: 0,
            buffer_capacity: 0,
            is_done: true,
        }
    }

    /// 今回のロードでシーケンス末尾に追加されたデータの量
    pub fn commited(&self) -> usize {
        self.commited
    }
    /// ソースから読み込まれたがシーケンスにコミットされていないデータの量。`buffer_capacity`以下になる。
    ///
    ///  `is_done`が`true`の場合、`uncommited`と`buffer_capacity`の値は`0`になる。
    pub fn uncommited(&self) -> usize {
        self.uncommited
    }
    /// ローダー内部に確保しているバッファの容量。`uncommited`以上になる。
    ///
    ///  `is_done`が`true`の場合、`uncommited`と`buffer_capacity`の値は`0`になる。
    pub fn buffer_capacity(&self) -> usize {
        self.buffer_capacity
    }

    /// `true`の場合、ソース末尾に到達しており、これ以降データが追加されることはない。
    pub fn is_done(&self) -> bool {
        self.is_done
    }
}
