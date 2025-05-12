use std::collections::VecDeque;

use parcom_core::Stream;
use parcom_streams_core::StreamSource;

pub struct SingleBufferStream<T, S: StreamSource<Segment = [T]>> {
    buf: VecDeque<T>,
    source: S,
}

// impl<T, S: StreamSource<Segment = [T]>> Stream for SingleBufferStream<T, S> {
//     type Segment = [T];
//     type Error = S::Error;
//     type SegmentIter;
//     type Advance;

//     fn segments(&mut self) -> Self::SegmentIter {
//         todo!()
//     }

//     fn advance(
//         self,
//         delta: <Self::Segment as parcom_core::StreamSegment>::Length,
//     ) -> Self::Advance {
//         todo!()
//     }
// }
