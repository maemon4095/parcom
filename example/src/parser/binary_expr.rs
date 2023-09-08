#![cfg_attr(test, cfg(test))]

use parcom::foreign::parser::str::atom;
use parcom::standard::binary_expr::*;
use parcom::standard::parser::binary_expr::BinaryExprParser;
use parcom::standard::ParserExtension;
use parcom::ParseResult::{self, *};
use parcom::Parser;
use parcom::*;
/// parsing binary expression example. parse and eval expression with syntax below
/// expr = expr op expr / term
/// term = integer / (expr)
#[cfg_attr(test, test)]
pub fn main() {
    println!("----- binary expression example -----\n");

    let input = "1 + 2 * (6 + 4) / 5";

    println!(" input: {}", &input);

    let result = expr(input);

    let expr = match result {
        Done(expr, rest) => {
            println!("  rest: {}", rest);
            expr
        }
        Fail(_, rest) => {
            println!("error; rest: {}", rest);
            return;
        }
    };

    println!("result: {} = {}", display(&expr), eval(&expr));

    println!();
}

/// expr = expr op expr / term
fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
    BinaryExprParser::new(
        term.map(|t| Expr::Term(t)),
        space.join(op).join(space).map(|((_, op), _)| op),
    )
    .parse(input)
}

/// term = integer / (expr)
fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Term, ()> {
    integer
        .or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
        .map(|e| match e {
            standard::Either::First(n) => Term::Integer(n),
            standard::Either::Last(e) => Term::Parenthesized(Box::new(e)),
        })
        .map_err(|_| ())
        .parse(input)
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
    type Expr = Expr;
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

    fn construct(self, lhs: Self::Expr, rhs: Self::Expr) -> Self::Expr {
        Expr::BinOp(Box::new(lhs), self, Box::new(rhs))
    }
}

fn space<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, (), ()> {
    foreign::parser::str::atom_char(' ')
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
        _ => return Fail((), input),
    };

    Done(op, input.advance(1))
}

fn integer<S: Stream<Segment = str>>(input: S) -> ParseResult<S, usize, ()> {
    let chars = input.segments().flat_map(|e| e.chars());
    let radix = 10;

    let (max_digit, to_consume) = {
        let mut chars = chars.take_while(|c| c.is_digit(radix));
        if chars.next().is_none() {
            drop(chars);
            return Fail((), input);
        }

        let mut digit = 1;
        let mut consume = 1;

        for _ in chars {
            digit *= radix;
            consume += 1;
        }

        (digit, consume)
    };

    let chars = input.segments().flat_map(|e| e.chars());
    let mut sum = 0;
    let mut digit = max_digit;
    for c in chars {
        let Some(d) = c.to_digit(10) else {
            break;
        };

        sum += (d * digit) as usize;
        digit /= radix;
    }

    Done(sum, input.advance(to_consume))
}
