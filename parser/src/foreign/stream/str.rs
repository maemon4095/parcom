use crate::{ParseStream, RewindStream, Stream};

#[derive(Clone)]
pub struct StrStream<'me> {
    location: Location,
    str: &'me str,
}

impl<'me> StrStream<'me> {
    pub fn new(str: &'me str) -> Self {
        Self {
            location: Location {
                total_count: 0,
                line: 1,
                column: 0,
            },
            str,
        }
    }

    fn loc(&self, count: usize) -> Location {
        self.str
            .chars()
            .take(count)
            .fold((false, self.location.clone()), |(mut newline, mut l), c| {
                l.total_count += 1;

                if newline {
                    l.column = 1;
                    l.line += 1;
                    newline = false;
                } else {
                    l.column += 1;
                }

                if c == '\n' {
                    newline = true;
                }

                (newline, l)
            })
            .1
    }
}

impl<'me> Stream for StrStream<'me> {
    type Segment = str;

    type Iter<'a> = std::iter::Once<&'a str>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self.str)
    }

    fn advance(mut self, count: usize) -> Self {
        self.location = self.loc(count);
        self.str = &self.str[count..];
        self
    }
}
impl<'me> RewindStream for StrStream<'me> {
    type Anchor = Self;

    fn anchor(&self) -> Self::Anchor {
        self.clone()
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor
    }
}
impl<'me> ParseStream for StrStream<'me> {
    type Location = Location;

    fn location(&self, index: usize) -> Self::Location {
        self.loc(index + 1)
    }
}

#[derive(Clone, Eq, Debug)]
pub struct Location {
    total_count: usize,
    line: usize,
    column: usize,
}

impl crate::Location for Location {
    fn distance(&self, rhs: &Self) -> usize {
        if self.total_count < rhs.total_count {
            rhs.total_count - self.total_count
        } else {
            self.total_count - rhs.total_count
        }
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.total_count == other.total_count
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_count.cmp(&other.total_count)
    }
}

impl Location {
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

#[cfg(test)]
mod test {
    use crate::{ParseStream, Stream};

    use super::StrStream;

    #[test]
    fn advance() {
        let stream = StrStream::new("aaaa");
        println!("{:?}", stream.location(0));
        let stream = stream.advance(0);
        println!("{:?}", stream.location(0));
    }
}
