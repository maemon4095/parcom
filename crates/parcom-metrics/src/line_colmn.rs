use parcom_core::{Meter, Metrics};

#[derive(Debug, Eq, Ord, Clone)]
pub struct LineColumn {
    pub total_count: usize,
    pub line: usize,
    pub column: usize,
}

impl PartialEq for LineColumn {
    fn eq(&self, other: &Self) -> bool {
        self.total_count == other.total_count
    }
}

impl PartialOrd for LineColumn {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.total_count.cmp(&other.total_count))
    }
}

impl std::default::Default for LineColumn {
    fn default() -> Self {
        Self {
            total_count: 0,
            line: 0,
            column: 0,
        }
    }
}

impl Metrics<str> for LineColumn {
    type Meter = LineColumnMeter;
}

#[derive(Debug, Clone)]
pub struct LineColumnMeter {
    location: LineColumn,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Initial,
    CR,
}

impl Meter<str> for LineColumnMeter {
    type Metrics = LineColumn;
    fn metrics(&self) -> Self::Metrics {
        self.location.clone()
    }

    fn advance(mut self, segment: &str) -> Self {
        for c in segment.chars() {
            match c {
                '\r' => {
                    self.state = State::CR;
                    self.location.line += 1;
                    self.location.column = 0;
                }
                '\n' => match self.state {
                    State::Initial => {
                        self.location.line += 1;
                        self.location.column = 0;
                    }
                    State::CR => {
                        // CRLFが連続で並んでいる場合lineに加算しない; CRを見つけた時点でlineに加算するため。
                        self.state = State::Initial;
                    }
                },
                _ => {
                    self.state = State::Initial;
                    self.location.column += 1;
                }
            }
            self.location.total_count += 1;
        }

        self
    }
}

impl LineColumnMeter {
    pub fn new(start: LineColumn) -> Self {
        Self {
            location: start,
            state: State::Initial,
        }
    }
}

impl Default for LineColumnMeter {
    fn default() -> Self {
        Self {
            location: Default::default(),
            state: State::Initial,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let cases = ["", "oneline", "line0\n\r\rline3\r\nline4\n"];

        for str in cases {
            let zero = LineColumnMeter::default();

            assert_eq!(
                zero.advance(str).metrics(),
                LineColumn {
                    total_count: str.len(),
                    line: str.lines().count(),
                    column: str.lines().last().map_or(0, |l| l.len())
                }
            )
        }
    }
}
