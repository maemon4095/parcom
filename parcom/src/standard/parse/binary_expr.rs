use std::marker::PhantomData;

use crate::{ParseResult, RewindStream};

use crate::Parse;

use crate::standard::binary_expr::{Associativity, Operator};

pub struct BinaryExpr<Expr, Term, Op>
where
    Expr: From<Term>,
    Op: Operator<Expr = Expr>,
{
    pub expr: Expr,
    marker: PhantomData<(Term, Op)>,
}

impl<S, Expr, Term, Op> Parse<S> for BinaryExpr<Expr, Term, Op>
where
    S: RewindStream,
    Expr: From<Term>,
    Term: Parse<S>,
    Op: Parse<S> + Operator<Expr = Expr>,
{
    type Error = Term::Error;

    fn parse(input: S) -> crate::ParseResult<S, Self, Self::Error> {
        let (expr, rest) = parse_impl::<S, Expr, Term, Op>(input, 0, 256)?;
        let me = Self {
            expr,
            marker: PhantomData,
        };
        return Ok((me, rest));

        fn parse_impl<S, Expr, Term, Op>(
            input: S,
            precedence: usize,
            recursion_limit: usize,
        ) -> ParseResult<S, Expr, Term::Error>
        where
            S: RewindStream,
            Expr: From<Term>,
            Term: Parse<S>,
            Op: Parse<S> + Operator<Expr = Expr>,
        {
            if recursion_limit == 0 {
                return parse_impl_loop::<S, Expr, Term, Op>(input, precedence);
            }

            let (term, mut rest) = Term::parse(input)?;
            let mut lhs = Expr::from(term);
            loop {
                let anchor = rest.anchor();
                let (op, r) = match Op::parse(rest) {
                    Ok((op, r)) if op.precedence() >= precedence => (op, r),
                    Ok((_, r)) | Err((_, r)) => {
                        rest = r.rewind(anchor);
                        break;
                    }
                };

                let next_prec = match op.associativity() {
                    Associativity::Left => op.precedence() + 1,
                    Associativity::Right => op.precedence(),
                };

                let (rhs, r) = parse_impl::<S, Expr, Term, Op>(r, next_prec, recursion_limit - 1)?;

                lhs = op.construct(lhs, rhs);
                rest = r;
            }

            Ok((lhs, rest))
        }

        // consider the input has the syntax "term / (term op)+ term"
        fn parse_impl_loop<S, Expr, Term, Op>(
            input: S,
            precedence: usize,
        ) -> ParseResult<S, Expr, Term::Error>
        where
            S: RewindStream,
            Expr: From<Term>,
            Term: Parse<S>,
            Op: Parse<S> + Operator<Expr = Expr>,
        {
            let (term, rest) = Term::parse(input)?;
            let lhs = Expr::from(term);

            let (op, mut rest) = {
                let anchor = rest.anchor();
                match Op::parse(rest) {
                    Ok((op, r)) if op.precedence() >= precedence => (op, r),
                    Ok((_, r)) | Err((_, r)) => return Ok((lhs, r.rewind(anchor))),
                }
            };

            let mut stack = vec![(lhs, op)];

            let mut rhs = loop {
                let prec = stack.last().map(|(_, op)| next_precedence(op)).unwrap();
                let (term, r) = Term::parse(rest)?;
                let term = Expr::from(term);

                let anchor = r.anchor();
                match Op::parse(r) {
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
                        break term;
                    }
                };
            };

            while let Some((lhs, op)) = stack.pop() {
                rhs = op.construct(lhs, rhs);
            }

            return Ok((rhs, rest));

            fn next_precedence<T: Operator>(op: &T) -> usize {
                match op.associativity() {
                    Associativity::Left => op.precedence() + 1,
                    Associativity::Right => op.precedence(),
                }
            }
        }
    }
}
