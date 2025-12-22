use parcom_core::{ParseError, ParseResult, Parser, ParserOnce, ParserResult, RewindSequence};
use parcom_internals::ShortVec;
use parcom_util::{done, fail, Either};
use std::marker::PhantomData;

pub enum Associativity {
    Left,
    Right,
}

pub trait Operator {
    fn precedence(&self) -> usize;
    fn associativity(&self) -> Associativity;
}

// https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
#[derive(Debug)]
pub struct BinExprParser<S: RewindSequence, PTerm: Parser<S>, POp: Parser<S>, Expr>
where
    POp::Output: Operator,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    parser_term: PTerm,
    parser_op: POp,
    marker: PhantomData<(S, Expr)>,
}

impl<S, PTerm, POp, Expr> ParserOnce<S> for BinExprParser<S, PTerm, POp, Expr>
where
    S: RewindSequence,
    POp::Output: Operator,
    PTerm: Parser<S>,
    POp: Parser<S>,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    type Output = (Expr, Either<POp::Error, PTerm::Error>);
    type Error = Either<POp::Error, PTerm::Error>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S, PTerm, POp, Expr> Parser<S> for BinExprParser<S, PTerm, POp, Expr>
where
    S: RewindSequence,
    POp::Output: Operator,
    PTerm: Parser<S>,
    POp: Parser<S>,
    Expr: From<(Expr, POp::Output, Expr)> + From<PTerm::Output>,
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let ((e, reason), rest) = self.parse_impl(input, 0).await?;
        // Ok になる場合は op をパースしたときに op.precedence() < precedence の場合のみ．
        // 常に precedence >= 0 であるから，ここで Ok にはならない．
        let Err(reason) = reason else { unreachable!() };
        done((e, reason), rest)
    }
}

impl<S, PTerm, POp, Expr> BinExprParser<S, PTerm, POp, Expr>
where
    S: RewindSequence,
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
        let (rhs, mut rest) = self
            .parser_term
            .parse(input)
            .await
            .map_err(|(e, r)| (Either::Last(e), r))?;
        let mut rhs = Expr::from(rhs);

        // (lhs0 op0 (lhs1 op1 ... (lhsN opN rhs
        // 優先度は op0 <= op1 <= ... <= opN
        let mut stack = ShortVec::<_, 4>::new();

        let reason = loop {
            let anchor = rest.anchor();
            let (op, r) = match self.parser_op.parse(rest).await {
                Ok((e, r)) if e.precedence() >= precedence => (e, r),
                Ok((e, r)) => {
                    rest = r;
                    break Ok((e, anchor));
                }
                Err((e, r)) if e.should_terminate() => return fail(Either::First(e), r),
                Err((e, r)) => {
                    rest = r.rewind(anchor).await;
                    break Err(Either::First(e));
                }
            };

            let (term, r) = match self.parser_term.parse(r).await {
                Ok((e, r)) => (Expr::from(e), r),
                Err((e, r)) if !e.should_terminate() => {
                    rest = r.rewind(anchor).await;
                    break Err(Either::Last(e));
                }
                Err((e, r)) => return fail(Either::Last(e), r),
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

        done((rhs, reason), rest)
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
    use crate::primitive::{any_char, the_char};
    use crate::{primitive::atom, ParserExtension};
    use parcom_core::{ParseResult, Parser, RewindSequence, Sequence};
    use parcom_util::{error::Miss, Either};

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
                Ok((_, r)) => {
                    assert!(r.is_empty());
                }
                _ => unreachable!(),
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
                Ok((_, r)) => {
                    assert_eq!(r, " ~ @@@");
                }
                _ => unreachable!(),
            }
        })
    }

    /// expr = expr op expr / term
    async fn expr<S: RewindSequence<Segment = str>>(input: S) -> ParseResult<S, Expr, Miss<()>> {
        BinExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op))
            .map(|(e, _)| e)
            .map_err(|_| ().into())
            .boxed()
            .parse(input)
            .await
    }

    /// term = 0 / (expr)
    async fn term<S: RewindSequence<Segment = str>>(input: S) -> ParseResult<S, Term, Miss<()>> {
        zero.or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
            .map(|e| match e {
                Either::First(_) => Term::Zero,
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
        Zero,
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

    async fn space<S: RewindSequence<Segment = str>>(input: S) -> ParseResult<S, (), Miss<()>> {
        the_char(' ')
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

    async fn op<S: Sequence<Segment = str>>(input: S) -> ParseResult<S, Op, Miss<()>> {
        any_char()
            .and_then(|head| {
                let op = match head {
                    '+' => Op::Add,
                    '-' => Op::Sub,
                    '*' => Op::Mul,
                    '/' => Op::Div,
                    '~' => Op::Til,
                    _ => return Err(Miss(())),
                };

                Ok(op)
            })
            .parse(input)
            .await
    }

    async fn zero<S: Sequence<Segment = str>>(input: S) -> ParseResult<S, (), Miss<()>> {
        atom("0").parse(input).await
    }
}
