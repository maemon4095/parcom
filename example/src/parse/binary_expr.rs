#![cfg_attr(test, cfg(test))]

use parcom::standard::binary_expr::Operator;
use parcom::{ParseResult, Parser, Stream};

use parcom::foreign::parser::str::atom_char;
use parcom::standard::parser::binary_expr::BinaryExprParser;
use parcom::{
    standard::{self, parse::ParseExtension, ParserExtension},
    Parse, RewindStream,
};

#[cfg_attr(test, test)]
pub fn main() {
    let result = Expr::parse("0+0*0/0+0");

    let (expr, rest) = match result {
        Ok(t) => t,
        Err((_, r)) => {
            println!("error; rest: {r}");
            return;
        }
    };

    println!("expr: {}", display(&expr));
    println!("rest: {}", rest);
}

fn display(expr: &Expr) -> String {
    match expr {
        Expr::Bin(l, op, r) => {
            let l = display(l);
            let r = display(r);
            match op {
                Op::Add => format!("({} + {})", l, r),
                Op::Sub => format!("({} - {})", l, r),
                Op::Mul => format!("({} * {})", l, r),
                Op::Div => format!("({} / {})", l, r),
            }
        }
        Expr::Atom(atom) => match atom {
            Atom::Parenthesized(e) => display(e),
            Atom::Zero(_) => format!("0"),
        },
    }
}

#[derive(Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl<S: Stream<Segment = str>> Parse<S> for Op {
    type Error = ();

    fn parse(input: S) -> ParseResult<S, Self, Self::Error> {
        let mut chars = input.segments().flat_map(|s| s.chars());
        'scope: {
            match chars.next() {
                Some(c) => {
                    let op = match c {
                        '+' => Op::Add,
                        '-' => Op::Sub,
                        '*' => Op::Mul,
                        '/' => Op::Div,
                        _ => break 'scope,
                    };

                    drop(chars);
                    return Ok((op, input.advance(1)));
                }
                _ => break 'scope,
            }
        }

        drop(chars);
        Err(((), input))
    }
}

impl Operator for Op {
    type Expr = Expr;

    fn construct(self, lhs: Self::Expr, rhs: Self::Expr) -> Self::Expr {
        Expr::Bin(Box::new(lhs), self, Box::new(rhs))
    }

    fn precedence(&self) -> usize {
        match self {
            Op::Add => 0,
            Op::Sub => 0,
            Op::Mul => 1,
            Op::Div => 1,
        }
    }

    fn associativity(&self) -> standard::binary_expr::Associativity {
        standard::binary_expr::Associativity::Left
    }
}

#[derive(Debug)]
enum Expr {
    Bin(Box<Expr>, Op, Box<Expr>),
    Atom(Atom),
}

impl<S: RewindStream<Segment = str>> Parse<S> for Expr {
    type Error = ();

    fn parse(input: S) -> ParseResult<S, Self, Self::Error> {
        BinaryExprParser::new(
            Atom::into_parser().map(|a| Expr::Atom(a)),
            Op::into_parser(),
        )
        .parse(input)
    }
}
#[derive(Debug)]
enum Atom {
    Parenthesized(Box<Expr>),
    Zero(Zero),
}

impl<S: RewindStream<Segment = str>> Parse<S> for Atom {
    type Error = ();

    fn parse(input: S) -> ParseResult<S, Self, Self::Error> {
        Zero::into_parser()
            .map(|z| Atom::Zero(z))
            .or(atom_char('(')
                .join(Expr::into_parser())
                .join(atom_char(')'))
                .map(|((_, e), _)| Atom::Parenthesized(Box::new(e))))
            .unify()
            .map_err(|_| ())
            .parse(input)
    }
}
#[derive(Debug)]
struct Zero {}

impl<S: RewindStream<Segment = str>> Parse<S> for Zero {
    type Error = ();

    fn parse(input: S) -> ParseResult<S, Self, Self::Error> {
        let mut chars = input.segments().flat_map(|s| s.chars());
        match chars.next() {
            Some('0') => {
                drop(chars);
                Ok((Self {}, input.advance(1)))
            }
            _ => {
                drop(chars);
                Err(((), input))
            }
        }
    }
}
