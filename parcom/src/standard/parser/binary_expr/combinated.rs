use std::marker::PhantomData;

use crate::{ParseResult, Parser, RewindStream};

use super::{Associativity, Operator};

pub struct BinaryExprParser<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>>
where
    POp::Output: Operator,
{
    parser_term: PTerm,
    parser_op: POp,
    recursion_limit: usize,
    marker: PhantomData<S>,
}

impl<S, E, PTerm, POp> Parser<S> for BinaryExprParser<S, PTerm, POp>
where
    S: RewindStream,
    POp::Output: Operator<Expr = E>,
    PTerm: Parser<S, Output = E>,
    POp: Parser<S>,
{
    type Output = E;
    type Error = PTerm::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        self.parse_impl(input, 0, self.recursion_limit)
    }
}

impl<S, E, PTerm, POp> BinaryExprParser<S, PTerm, POp>
where
    S: RewindStream,
    POp::Output: Operator<Expr = E>,
    PTerm: Parser<S, Output = E>,
    POp: Parser<S>,
{
    pub fn new(parser_term: PTerm, parser_op: POp) -> Self {
        Self::new_with_limit(256, parser_term, parser_op)
    }

    pub fn new_with_limit(recursion_limit: usize, parser_term: PTerm, parser_op: POp) -> Self {
        Self {
            parser_term,
            parser_op,
            recursion_limit,
            marker: PhantomData,
        }
    }

    fn parse_impl(
        &self,
        input: S,
        precedence: usize,
        recursion_limit: usize,
    ) -> ParseResult<S, E, PTerm::Error> {
        if recursion_limit == 0 {
            return self.parse_impl_loop(input, precedence);
        }

        let (mut lhs, mut rest) = match self.parser_term.parse(input) {
            Ok(t) => t,
            Err((e, r)) => return Err((e, r)),
        };

        loop {
            let anchor = rest.anchor();
            let (op, r) = match self.parser_op.parse(rest) {
                Ok((op, r)) if op.precedence() >= precedence => (op, r),
                Ok((_, r)) => {
                    rest = r.rewind(anchor);
                    break;
                }
                Err((_, r)) => {
                    rest = r.rewind(anchor);
                    break;
                }
            };

            let next_prec = match op.associativity() {
                Associativity::Left => op.precedence() + 1,
                Associativity::Right => op.precedence(),
            };

            let (rhs, r) = self.parse_impl(r, next_prec, recursion_limit - 1)?;

            lhs = op.construct(lhs, rhs);
            rest = r;
        }

        Ok((lhs, rest))
    }

    fn parse_impl_loop(&self, input: S, precedence: usize) -> ParseResult<S, E, PTerm::Error> {
        let (lhs, rest) = match self.parser_term.parse(input) {
            Ok(t) => t,
            Err((e, r)) => return Err((e, r)),
        };

        let anchor = rest.anchor();
        let (op, mut rest) = match self.parser_op.parse(rest) {
            Ok((op, r)) if op.precedence() >= precedence => (op, r),
            Ok((_, r)) => return Ok((lhs, r.rewind(anchor))),
            Err((_, r)) => return Ok((lhs, r.rewind(anchor))),
        };

        let mut stack = vec![(lhs, op)];

        let expr = loop {
            let prec = stack.last().map(|(_, op)| next_precedence(op)).unwrap();
            let (term, r) = match self.parser_term.parse(rest) {
                Ok(t) => t,
                Err(t) => return Err(t),
            };

            let anchor = r.anchor();
            match self.parser_op.parse(r) {
                Ok((op, r)) if op.precedence() >= prec => {
                    rest = r;
                    stack.push((term, op));
                }
                Ok((op, r)) => {
                    rest = r;
                    let (lhs, operator) = stack.pop().unwrap();
                    stack.push((operator.construct(lhs, term), op));
                    continue;
                }
                Err((_, r)) => {
                    rest = r.rewind(anchor);
                    let mut rhs = term;
                    while let Some((lhs, op)) = stack.pop() {
                        rhs = op.construct(lhs, rhs);
                    }
                    break rhs;
                }
            };
        };

        return Ok((expr, rest));

        fn next_precedence<T: Operator>(op: &T) -> usize {
            match op.associativity() {
                Associativity::Left => op.precedence() + 1,
                Associativity::Right => op.precedence(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Associativity, BinaryExprParser, Operator};
    use crate::foreign::parser::str::{self, atom};
    use crate::standard::{Either, ParserExtension};
    use crate::{ParseResult, Parser, RewindStream, Stream};

    #[test]
    fn test() {
        let input = {
            let mut s = "0".to_string();
            s.extend(std::iter::repeat(" ~ 0").take(4096));
            s
        };
        let _ = expr(input.as_str());
    }

    /// expr = expr op expr / term
    fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
        BinaryExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op)).parse(input)
    }

    /// term = 0 / (expr)
    fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
        zero.or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
            .map(|e| match e {
                Either::First(c) => Expr::Atom(c),
                Either::Last(e) => Expr::Parenthesized(Box::new(e)),
            })
            .map_err(|_| ())
            .parse(input)
    }

    #[derive(Debug, Clone)]
    enum Expr {
        BinOp(Box<Expr>, Op, Box<Expr>),
        Atom(char),
        Parenthesized(Box<Expr>),
    }

    #[derive(Debug, Clone)]
    enum Op {
        Add,
        Sub,
        Mul,
        Div,
        Til,
    }

    impl Operator for Op {
        type Expr = Expr;
        fn precedence(&self) -> usize {
            match self {
                Op::Add => 1,
                Op::Sub => 1,
                Op::Mul => 2,
                Op::Div => 2,
                Op::Til => 0,
            }
        }

        fn associativity(&self) -> Associativity {
            match self {
                Op::Til => Associativity::Right,
                _ => Associativity::Left,
            }
        }

        fn construct(self, lhs: Self::Expr, rhs: Self::Expr) -> Self::Expr {
            Expr::BinOp(Box::new(lhs), self, Box::new(rhs))
        }
    }
    fn space<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, (), ()> {
        str::atom_char(' ')
            .discard()
            .repeat(1..)
            .discard()
            .parse(input)
    }

    fn op<S: Stream<Segment = str>>(input: S) -> ParseResult<S, Op, ()> {
        let mut chars = input.segments().flat_map(|s| s.chars());
        let Some(head) = chars.next() else {
            drop(chars);
            return Err(((), input));
        };
        drop(chars);
        let op = match head {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            '~' => Op::Til,
            _ => return Err(((), input)),
        };

        Ok((op, input.advance(1)))
    }

    fn zero<S: Stream<Segment = str>>(input: S) -> ParseResult<S, char, ()> {
        str::atom_char('0').parse(input)
    }
}
