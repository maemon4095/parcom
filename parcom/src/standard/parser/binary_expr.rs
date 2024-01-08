use std::marker::PhantomData;

use crate::{standard::Either, ParseResult::*, Parser, ParserResult, RewindStream};

use crate::standard::binary_expr::{Associativity, Operator};

pub struct BinaryExprParser<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>, Expr>
where
    POp::Output: Operator,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    parser_term: PTerm,
    parser_op: POp,
    recursion_limit: usize,
    marker: PhantomData<(S, Expr)>,
}

impl<S, PTerm, POp, Expr> Parser<S> for BinaryExprParser<S, PTerm, POp, Expr>
where
    S: RewindStream,
    POp::Output: Operator,
    PTerm: Parser<S>,
    POp: Parser<S>,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    type Output = Expr;
    type Error = PTerm::Error;
    type Fault = Either<PTerm::Fault, POp::Fault>;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parse_impl(input, 0, self.recursion_limit)
    }
}

impl<S, PTerm, POp, Expr> BinaryExprParser<S, PTerm, POp, Expr>
where
    S: RewindStream,
    POp::Output: Operator,
    PTerm: Parser<S>,
    POp: Parser<S>,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
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

        let (lhs, mut rest) = match self.parser_term.parse(input) {
            Done(v, r) => (v, r),
            Fail(v, r) => return Fail(v, r),
            Fatal(e) => return Fatal(Either::First(e)),
        };

        let mut lhs = Expr::from(lhs);

        loop {
            let anchor = rest.anchor();
            let (op, r) = match self.parser_op.parse(rest) {
                Done(op, r) if op.precedence() >= precedence => (op, r),
                Done(_, r) => {
                    rest = r.rewind(anchor);
                    break;
                }
                Fail(_, r) => {
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

            lhs = Expr::from((lhs, op, rhs));
            rest = r;
        }

        Done(lhs, rest)
    }

    // consider the input has syntax "term / (term op)+ term"
    fn parse_impl_loop(&self, input: S, precedence: usize) -> ParserResult<S, Self> {
        let (lhs, rest) = match self.parser_term.parse(input) {
            Done(v, r) => (v, r),
            Fail(v, r) => return Fail(v, r),
            Fatal(e) => return Fatal(Either::First(e)),
        };
        let lhs = Expr::from(lhs);

        let (op, mut rest) = {
            let anchor = rest.anchor();
            match self.parser_op.parse(rest) {
                Done(op, r) if op.precedence() >= precedence => (op, r),
                Done(_, r) => return Done(lhs, r.rewind(anchor)),
                Fail(_, r) => return Done(lhs, r.rewind(anchor)),
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
                    stack.push((Expr::from(term), op));
                }
                Done(op, r) => {
                    rest = r;
                    let (lhs, operator) = stack.pop().unwrap();
                    stack.push((Expr::from((lhs, operator, Expr::from(term))), op));
                    continue;
                }
                Fail(_, r) => {
                    rest = r.rewind(anchor);
                    break Expr::from(term);
                }
                Fatal(e) => return Fatal(Either::Last(e)),
            };
        };

        while let Some((lhs, op)) = stack.pop() {
            rhs = Expr::from((lhs, op, rhs));
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
    fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Term, ()> {
        zero.or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
            .map(|e| match e {
                Either::First(c) => Term::Atom(c),
                Either::Last(e) => Term::Parenthesized(e),
            })
            .map_err(|_| ())
            .never_fault()
            .parse(input)
    }

    #[derive(Debug, Clone)]
    enum Expr {
        Term(Box<Term>),
        Bin(Box<Expr>, Op, Box<Expr>),
    }

    impl From<Term> for Expr {
        fn from(args: Term) -> Self {
            Expr::Term(Box::new(args))
        }
    }

    impl From<(Expr, Op, Expr)> for Expr {
        fn from((lhs, op, rhs): (Expr, Op, Expr)) -> Self {
            Expr::Bin(Box::new(lhs), op, Box::new(rhs))
        }
    }

    #[derive(Debug, Clone)]
    enum Term {
        Atom(char),
        Parenthesized(Expr),
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
            return Fail((), input.into());
        };
        drop(chars);
        let op = match head {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            '~' => Op::Til,
            _ => return Fail((), input.into()),
        };

        Done(op, input.advance(1))
    }

    fn zero<S: Stream<Segment = str>>(input: S) -> ParseResult<S, char, ()> {
        str::atom_char('0').parse(input)
    }
}
