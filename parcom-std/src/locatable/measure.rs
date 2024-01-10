use parcom_core::Location;

#[derive(Debug, Eq, Ord, Clone)]
pub struct LineColumn {
    total_count: usize,
    line: usize,
    column: usize,
}

impl LineColumn {
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
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

impl Location<str> for LineColumn {
    fn advance(mut self, segment: &str) -> Self {
        for c in segment.chars() {
            self.total_count += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self
    }
}

#[cfg(test)]
mod test {
    use parcom_core::Location;

    use super::LineColumn;

    #[test]
    fn test() {
        let cases = ["", "oneline", "line0\nline1\n"];

        for str in cases {
            let zero = LineColumn::default();

            assert_eq!(
                zero.advance(str),
                LineColumn {
                    total_count: str.len(),
                    line: str.lines().count(),
                    column: str.lines().last().map_or(0, |l| l.len())
                }
            )
        }
    }
}
