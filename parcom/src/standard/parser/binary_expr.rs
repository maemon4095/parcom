use std::marker::PhantomData;

use crate::{standard::Either, ParseResult::*, Parser, ParserResult, RewindStream};

use crate::standard::binary_expr::{Associativity, Operator};

pub struct BinaryExprParser<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>>
where
    POp::Output: Operator<Expr = PTerm::Output>,
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
    type Fault = Either<PTerm::Fault, POp::Fault>;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
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
    ) -> ParserResult<S, Self> {
        if recursion_limit == 0 {
            return self.parse_impl_loop(input, precedence);
        }

        let (mut lhs, mut rest) = match self.parser_term.parse(input) {
            Done(v, r) => (v, r),
            Fail(v, r) => return Fail(v, r),
            Fatal(e) => return Fatal(Either::First(e)),
        };

        loop {
            let anchor = rest.anchor();
            let (op, r) = match self.parser_op.parse(rest) {
                Done(op, r) if op.precedence() >= precedence => (op, r),
                Done(_, r) | Fail(_, r) => {
                    rest = r.rewind(anchor);
                    break;
                }
                Fatal(e) => return Fatal(Either::Last(e)),
            };

            let next_prec = match op.associativity() {
                Associativity::Left => op.precedence() + 1,
                Associativity::Right => op.precedence(),
            };

            let (rhs, r) = match self.parse_impl(r, next_prec, recursion_limit - 1) {
                Done(v, r) => (v, r),
                Fail(v, r) => return Fail(v, r),
                Fatal(e) => return Fatal(e),
            };

            lhs = op.construct(lhs, rhs);
            rest = r;
        }

        Done(lhs, rest)
    }

    // consider the input has the syntax "term / (term op)+ term"
    fn parse_impl_loop(&self, input: S, precedence: usize) -> ParserResult<S, Self> {
        let (lhs, rest) = match self.parser_term.parse(input) {
            Done(v, r) => (v, r),
            Fail(v, r) => return Fail(v, r),
            Fatal(e) => return Fatal(Either::First(e)),
        };

        let (op, mut rest) = {
            let anchor = rest.anchor();
            match self.parser_op.parse(rest) {
                Done(op, r) if op.precedence() >= precedence => (op, r),
                Done(_, r) | Fail(_, r) => return Done(lhs, r.rewind(anchor)),
                Fatal(e) => return Fatal(Either::Last(e)),
            }
        };

        let mut stack = vec![(lhs, op)];

        let mut rhs = loop {
            let prec = stack.last().map(|(_, op)| next_precedence(op)).unwrap();
            let (term, r) = match self.parser_term.parse(rest) {
                Done(v, r) => (v, r),
                Fail(v, r) => return Fail(v, r),
                Fatal(e) => return Fatal(Either::First(e)),
            };

            let anchor = r.anchor();
            match self.parser_op.parse(r) {
                Done(op, r) if op.precedence() >= prec => {
                    rest = r;
                    stack.push((term, op));
                }
                Done(op, r) => {
                    rest = r;
                    let (lhs, operator) = stack.pop().unwrap();
                    stack.push((operator.construct(lhs, term), op));
                    continue;
                }
                Fail(_, r) => {
                    rest = r.rewind(anchor);
                    break term;
                }
                Fatal(e) => return Fatal(Either::Last(e)),
            };
        };

        while let Some((lhs, op)) = stack.pop() {
            rhs = op.construct(lhs, rhs);
        }

        return Done(rhs, rest);

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
    use parcom_core::Never;

    use super::{Associativity, BinaryExprParser, Operator};
    use crate::foreign::parser::str::{self, atom};
    use crate::standard::{parser::ParserExtension, Either};
    use crate::{
        ParseResult::{self, *},
        Parser, RewindStream, Stream,
    };

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
        BinaryExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op))
            .never_fault()
            .parse(input)
    }

    /// term = 0 / (expr)
    fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
        zero.or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
            .map(|e| match e {
                Either::First(c) => Expr::Atom(c),
                Either::Last(e) => Expr::Parenthesized(Box::new(e)),
            })
            .map_err(|_| ())
            .never_fault()
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
            return Fail((), input);
        };
        drop(chars);
        let op = match head {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            '~' => Op::Til,
            _ => return Fail((), input),
        };

        Done(op, input.advance(1))
    }

    fn zero<S: Stream<Segment = str>>(input: S) -> ParseResult<S, char, ()> {
        str::atom_char('0').parse(input)
    }
}
