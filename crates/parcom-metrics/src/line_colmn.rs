use crate::location;
use parcom_core::Metrics;

#[derive(Debug, Clone)]
pub struct LineColumn {
    location: location::LineColumn,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Initial,
    CR,
}

impl Metrics<str> for LineColumn {
    type Location = location::LineColumn;

    fn location(&self) -> Self::Location {
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

impl LineColumn {
    pub fn new(start: location::LineColumn) -> Self {
        Self {
            location: start,
            state: State::Initial,
        }
    }
}

impl Default for LineColumn {
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
    use parcom_core::Metrics;

    #[test]
    fn test() {
        let cases = ["", "oneline", "line0\n\r\rline3\r\nline4\n"];

        for str in cases {
            let zero = LineColumn::default();

            assert_eq!(
                zero.advance(str).location(),
                location::LineColumn {
                    total_count: str.len(),
                    line: str.lines().count(),
                    column: str.lines().last().map_or(0, |l| l.len())
                }
            )
        }
    }
}
