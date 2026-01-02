#![cfg_attr(test, cfg(test))]

use error::Miss;
use parcom::{
    parsers::{
        bin_expr::{Associativity, BinExprParser, Operator},
        primitive::{atom, the_char},
    },
    prelude::*,
    primitive::BytesDelta,
};
/// parsing binary expression example. parse and eval expression with syntax below
/// expr = expr op expr / term
/// term = integer / (expr)
#[cfg_attr(test, test)]
pub fn main() {
    pollster::block_on(async {
        println!("----- binary expression example -----\n");

        let input = "1 + 2 * (6 + 4) / 5";

        println!(" input: {}", &input);

        let result = expr(input).await;

        let expr = match result {
            Ok((expr, rest)) => {
                println!("  rest: {}", rest);
                expr
            }
            Err((_, rest)) => unsafe {
                println!("error; rest: {}", rest.unwrap());
                return;
            },
        };

        println!("result: {} = {}", display(&expr), eval(&expr));

        println!();
    })
}

/// expr = expr op expr / term
async fn expr<S: RewindSequence<Segment = str, Length = BytesDelta>>(
    input: S,
) -> ParseResult<S, Expr, Miss<()>> {
    BinExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op))
        .map(|(e, _)| e)
        .map_err(|_| ().into())
        .boxed()
        .parse(input)
        .await
}

/// term = integer / (expr)
async fn term<S: RewindSequence<Segment = str, Length = BytesDelta>>(
    input: S,
) -> ParseResult<S, Term, Miss<()>> {
    integer
        .or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
        .map(|e| match e {
            Either::First(n) => Term::Integer(n),
            Either::Last(e) => Term::Parenthesized(Box::new(e)),
        })
        .map_err(|_| ().into())
        .parse(input)
        .await
}

fn display(expr: &Expr) -> String {
    match expr {
        Expr::BinOp(l, op, r) => {
            let l = display(l);
            let r = display(r);
            match op {
                Op::Add => format!("({l} + {r})"),
                Op::Sub => format!("({l} - {r})"),
                Op::Mul => format!("({l} * {r})"),
                Op::Div => format!("({l} / {r})"),
            }
        }
        Expr::Term(atom) => match atom {
            Term::Parenthesized(e) => format!("{}", display(e)),
            Term::Integer(n) => format!("{n}"),
        },
    }
}

fn eval(expr: &Expr) -> usize {
    match expr {
        Expr::BinOp(l, op, r) => {
            let l = eval(l);
            let r = eval(r);
            match op {
                Op::Add => l + r,
                Op::Sub => l - r,
                Op::Mul => l * r,
                Op::Div => l / r,
            }
        }
        Expr::Term(term) => match term {
            Term::Parenthesized(e) => eval(e),
            Term::Integer(n) => *n,
        },
    }
}

#[derive(Debug)]
enum Expr {
    BinOp(Box<Expr>, Op, Box<Expr>),
    Term(Term),
}

impl From<Term> for Expr {
    fn from(args: Term) -> Self {
        Expr::Term(args)
    }
}

impl From<(Expr, Op, Expr)> for Expr {
    fn from((lhs, op, rhs): (Expr, Op, Expr)) -> Self {
        Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
    }
}

#[derive(Debug)]
enum Term {
    Parenthesized(Box<Expr>),
    Integer(usize),
}

#[derive(Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operator for Op {
    fn precedence(&self) -> usize {
        match self {
            Op::Add => 1,
            Op::Sub => 1,
            Op::Mul => 2,
            Op::Div => 2,
        }
    }

    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
}

async fn space<S: RewindSequence<Segment = str, Length = BytesDelta>>(
    input: S,
) -> ParseResult<S, (), Miss<()>> {
    the_char(' ')
        .repeat()
        .and_then(|(v, _)| if v.is_empty() { Err(Miss(())) } else { Ok(()) })
        .parse(input)
        .await
}

async fn op<S: Sequence<Segment = str, Length = BytesDelta>>(
    mut input: S,
) -> ParseResult<S, Op, Miss<()>> {
    let head = {
        let mut segments = input.segments();

        loop {
            let Some(segment) = segments.next(BytesDelta::from_bytes(0)).await else {
                drop(segments);
                return fail((), input);
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
        _ => return fail((), input),
    };

    done(op, input.advance(BytesDelta::from_char(head)).await)
}

async fn integer<S: Sequence<Segment = str, Length = BytesDelta>>(
    mut input: S,
) -> ParseResult<S, usize, Miss<()>> {
    let mut segments = input.segments();
    let mut buf = String::new();

    let mut consumed_bytes = 0;
    while let Some(segment) = segments.next(BytesDelta::ZERO).await {
        let c = segment
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit())
            .last();

        match c {
            Some((idx, c)) => {
                let consumed = idx + c.len_utf8();
                consumed_bytes += consumed;

                buf.push_str(&segment[..consumed]);
                if consumed < segment.len() {
                    break;
                }
            }
            None => break,
        }
    }

    if consumed_bytes == 0 {
        drop(segments);
        return fail((), input);
    }
    let n = usize::from_str_radix(&buf, 10).unwrap();

    drop(segments);
    done(
        n,
        input.advance(BytesDelta::from_bytes(consumed_bytes)).await,
    )
}
