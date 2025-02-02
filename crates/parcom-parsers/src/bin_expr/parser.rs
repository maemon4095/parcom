use super::{Associativity, Operator};
use parcom_base::Either;
use parcom_core::{
    ParseError,
    ParseResult::{self, *},
    Parser, ParserResult, RewindStream,
};
use parcom_internals::ShortVec;
use std::marker::PhantomData;

// https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
#[derive(Debug)]
pub struct BinExprParser<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>, Expr>
where
    POp::Output: Operator,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    parser_term: PTerm,
    parser_op: POp,
    marker: PhantomData<(S, Expr)>,
}

impl<S, PTerm, POp, Expr> Parser<S> for BinExprParser<S, PTerm, POp, Expr>
where
    S: RewindStream,
    POp::Output: Operator,
    PTerm: Parser<S>,
    POp: Parser<S>,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    type Output = (Expr, Either<POp::Error, PTerm::Error>);
    type Error = Either<POp::Error, PTerm::Error>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let result = self.parse_impl(input, 0).await;

        match result {
            Done((e, reason), rest) => {
                // Ok になる場合は op をパースしたときに op.precedence() < precedence の場合のみ．
                // 常に precedence >= 0 であるから，ここで Ok にはならない．
                let Err(reason) = reason else { unreachable!() };
                Done((e, reason), rest)
            }
            Fail(e, r) => Fail(e, r),
        }
    }
}

impl<S, PTerm, POp, Expr> BinExprParser<S, PTerm, POp, Expr>
where
    S: RewindStream,
    POp::Output: Operator,
    PTerm: Parser<S>,
    POp: Parser<S>,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    pub fn new(parser_term: PTerm, parser_op: POp) -> Self {
        Self {
            parser_term,
            parser_op,
            marker: PhantomData,
        }
    }

    // consider the input has syntax "term / (term op)+ term"
    async fn parse_impl(
        &self,
        input: S,
        precedence: usize,
    ) -> ParseResult<
        S,
        (
            Expr,
            Result<(POp::Output, S::Anchor), Either<POp::Error, PTerm::Error>>,
        ),
        Either<POp::Error, PTerm::Error>,
    > {
        let (rhs, mut rest) = match self.parser_term.parse(input).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::Last(e), r),
        };
        let mut rhs = Expr::from(rhs);

        // (lhs0 op0 (lhs1 op1 ... (lhsN opN rhs
        // 優先度は op0 <= op1 <= ... <= opN
        let mut stack = ShortVec::<_, 8>::new();

        let reason = loop {
            let anchor = rest.anchor();
            let (op, r) = match self.parser_op.parse(rest).await {
                Done(e, r) if e.precedence() >= precedence => (e, r),
                Done(e, r) => {
                    rest = r;
                    break Ok((e, anchor));
                }
                Fail(e, r) if e.should_terminate() => return Fail(Either::First(e), r),
                Fail(e, r) => {
                    rest = r.rewind(anchor).await;
                    break Err(Either::First(e));
                }
            };

            let (term, r) = match self.parser_term.parse(r).await {
                Done(e, r) => (Expr::from(e), r),
                Fail(e, r) if !e.should_terminate() => {
                    rest = r.rewind(anchor).await;
                    break Err(Either::Last(e));
                }
                Fail(e, r) => return Fail(Either::Last(e), r),
            };

            rest = r;

            loop {
                let Some((lhs, last_op)) = stack.pop() else {
                    stack.push((rhs, op));
                    rhs = term;
                    break;
                };
                let prec = next_precedence(&last_op);
                if op.precedence() >= prec {
                    // ひとつ前の演算子より優先度が高い．
                    stack.push((lhs, last_op));
                    stack.push((rhs, op));
                    rhs = term;
                    break;
                }

                // ひとつ前の演算子より優先度が低い．
                rhs = Expr::from((lhs, last_op, rhs));
            }
        };

        while let Some((lhs, op)) = stack.pop() {
            rhs = Expr::from((lhs, op, rhs));
        }

        Done((rhs, reason), rest)
    }
}
fn next_precedence<T: Operator>(op: &T) -> usize {
    match op.associativity() {
        Associativity::Left => op.precedence() + 1,
        Associativity::Right => op.precedence(),
    }
}

#[cfg(test)]
mod test {
    use super::{Associativity, BinExprParser, Operator};
    use crate::{
        primitive::str::{self, atom},
        ParserExtension,
    };
    use parcom_base::{error::Miss, Either};
    use parcom_core::SegmentIterator;
    use parcom_core::{
        ParseResult::{self, *},
        Parser, RewindStream, Stream,
    };

    #[test]
    fn successful_input_and_no_chars_left() {
        pollster::block_on(async {
            let input = {
                let mut s = "0".to_string();
                s.extend(std::iter::repeat(" ~ 0").take(8192));
                s
            };
            let result = expr(input.as_str()).await;

            match result {
                Done(_, r) => {
                    assert!(r.is_empty());
                }
                Fail(_, _) => unreachable!(),
            }
        })
    }

    #[test]
    fn successful_input_and_with_chars_left() {
        pollster::block_on(async {
            let input = {
                let mut s = "0".to_string();
                s.extend(std::iter::repeat(" ~ 0").take(8192));
                s.push_str(" ~ @@@");
                s
            };
            let result = expr(input.as_str()).await;

            match result {
                Done(_, r) => {
                    assert_eq!(r, " ~ @@@");
                }
                Fail(_, _) => unreachable!(),
            }
        })
    }

    /// expr = expr op expr / term
    async fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, Miss<()>> {
        BinExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op))
            .map(|(e, _)| e)
            .map_err(|_| ().into())
            .boxed()
            .parse(input)
            .await
    }

    /// term = 0 / (expr)
    async fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Term, Miss<()>> {
        zero.or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
            .map(|e| match e {
                Either::First(c) => Term::Atom(c),
                Either::Last(e) => Term::Parenthesized(e),
            })
            .map_err(|_| ().into())
            .parse(input)
            .await
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    async fn space<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, (), Miss<()>> {
        str::atom_char(' ')
            .repeat()
            .and_then(|e| {
                if e.0.is_empty() {
                    Err(Miss(()))
                } else {
                    Ok(())
                }
            })
            .parse(input)
            .await
    }

    async fn op<S: Stream<Segment = str>>(input: S) -> ParseResult<S, Op, Miss<()>> {
        let head = {
            let mut segments = input.segments();

            loop {
                let Some(segment) = segments.next(0).await else {
                    return Fail(().into(), input.into());
                };

                if let Some(c) = segment.chars().next() {
                    break c;
                }
            }
        };

        let op = match head {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            '~' => Op::Til,
            _ => return Fail(().into(), input.into()),
        };

        Done(op, input.advance(1.into()).await)
    }

    async fn zero<S: Stream<Segment = str>>(input: S) -> ParseResult<S, char, Miss<()>> {
        str::atom_char('0').parse(input).await
    }
}
