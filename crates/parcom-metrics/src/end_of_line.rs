#[derive(Debug, PartialEq, Eq)]
pub enum EndOfLineMatch {
    NotMatch,
    PartialMatch,
    Match,
}

pub trait EndOfLine: Default {
    fn next(&mut self, c: char) -> EndOfLineMatch;
}

#[derive(Debug, Default)]
pub struct LF;

impl EndOfLine for LF {
    fn next(&mut self, c: char) -> EndOfLineMatch {
        match c {
            '\n' => EndOfLineMatch::Match,
            _ => EndOfLineMatch::NotMatch,
        }
    }
}

#[derive(Debug, Default)]
pub struct CR;

impl EndOfLine for CR {
    fn next(&mut self, c: char) -> EndOfLineMatch {
        match c {
            '\r' => EndOfLineMatch::Match,
            _ => EndOfLineMatch::NotMatch,
        }
    }
}

#[derive(Debug, Default)]
enum CRLFState {
    #[default]
    Initial,
    CR,
}

#[derive(Debug, Default)]
pub struct CRLF {
    state: CRLFState,
}

impl EndOfLine for CRLF {
    fn next(&mut self, c: char) -> EndOfLineMatch {
        match self.state {
            CRLFState::Initial => match c {
                '\r' => {
                    self.state = CRLFState::CR;
                    EndOfLineMatch::PartialMatch
                }
                _ => EndOfLineMatch::NotMatch,
            },
            CRLFState::CR => {
                self.state = CRLFState::Initial;
                match c {
                    '\n' => EndOfLineMatch::Match,
                    _ => EndOfLineMatch::NotMatch,
                }
            }
        }
    }
}
