use crate::end_of_line::{self, EndOfLine, EndOfLineMatch};
use parcom_core::measured::{Meter, Metrics};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct LineColumn<EOL: EndOfLine = end_of_line::LF> {
    pub total_count: usize,
    pub line: usize,
    pub column: usize,
    marker: PhantomData<EOL>,
}

impl<EOL: EndOfLine> Default for LineColumn<EOL> {
    fn default() -> Self {
        Self {
            total_count: 0,
            line: 0,
            column: 0,
            marker: PhantomData,
        }
    }
}

impl<EOL: EndOfLine> Clone for LineColumn<EOL> {
    fn clone(&self) -> Self {
        Self {
            total_count: self.total_count.clone(),
            line: self.line.clone(),
            column: self.column.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<EOL: EndOfLine> Eq for LineColumn<EOL> {}
impl<EOL: EndOfLine> PartialEq for LineColumn<EOL> {
    fn eq(&self, other: &Self) -> bool {
        self.total_count == other.total_count
            && self.line == other.line
            && self.column == other.column
    }
}

impl<EOL: EndOfLine> Ord for LineColumn<EOL> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl<EOL: EndOfLine> PartialOrd for LineColumn<EOL> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.total_count.cmp(&other.total_count))
    }
}

impl<EOL: EndOfLine> Metrics<str> for LineColumn<EOL> {
    type Meter = LineColumnMeter<EOL>;
}

#[derive(Debug)]
pub struct LineColumnMeter<EOL: EndOfLine> {
    location: LineColumn<EOL>,
    next_is_new_line: bool,

    state: EOL,
}

impl<EOL: EndOfLine> Meter<str> for LineColumnMeter<EOL> {
    type Metrics = LineColumn<EOL>;
    fn metrics(&self) -> Self::Metrics {
        self.location.clone()
    }

    fn advance(mut self, segment: &str) -> Self {
        for c in segment.chars() {
            if self.next_is_new_line {
                self.location.line += 1;
                self.location.column = 0;
                self.next_is_new_line = false;
            }
            self.location.total_count += 1;
            self.location.column += 1;

            if let EndOfLineMatch::Match = self.state.next(c) {
                self.next_is_new_line = true;
            }
        }

        self
    }
}

impl<EOL: EndOfLine> LineColumnMeter<EOL> {
    pub fn new(start: LineColumn<EOL>) -> Self {
        Self {
            location: start,
            next_is_new_line: true,
            state: Default::default(),
        }
    }
}

impl<EOL: EndOfLine> Default for LineColumnMeter<EOL> {
    fn default() -> Self {
        Self {
            location: Default::default(),
            next_is_new_line: true,
            state: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_lf_without_ending_lf() {
        let cases = ["", "oneline", "\r\r", "line0\n\r\rline3\r\nline4"];

        for str in cases {
            let zero = LineColumnMeter::default();

            assert_eq!(
                zero.advance(str).metrics(),
                LineColumn::<end_of_line::LF> {
                    total_count: str.len(),
                    line: str.lines().count(),
                    column: str.lines().last().map_or(0, |l| l.len()),
                    marker: PhantomData
                },
                "case: {}",
                str
            )
        }
    }

    #[test]
    fn test_lf_with_ending_lf() {
        let cases = ["\n", "oneline\n", "line0\n\r\rline3\nline4\n"];

        for str in cases {
            let zero = LineColumnMeter::default();

            assert_eq!(
                zero.advance(str).metrics(),
                LineColumn::<end_of_line::LF> {
                    total_count: str.len(),
                    line: str.lines().count(),
                    column: str.lines().last().map_or(0, |l| l.len() + 1),
                    marker: PhantomData
                },
                "case: {}",
                str
            )
        }
    }

    #[test]
    fn test_lf_with_ending_crlf() {
        let cases = ["\r\n", "oneline\r\n", "line0\n\r\rline3\nline4\r\n"];

        for str in cases {
            let zero = LineColumnMeter::default();

            assert_eq!(
                zero.advance(str).metrics(),
                LineColumn::<end_of_line::LF> {
                    total_count: str.len(),
                    line: str.lines().count(),
                    column: str.lines().last().map_or(0, |l| l.len() + 2),
                    marker: PhantomData
                },
                "case: {}",
                str
            )
        }
    }
}
