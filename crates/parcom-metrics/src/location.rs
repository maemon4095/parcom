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
