#![cfg_attr(test, cfg(test))]

use parcom::{
    prelude::*,
    std::{
        binary_expr::{Associativity, BinaryExprParser, Operator},
        primitive::str::atom_char,
        ParserExtension,
    },
};

#[cfg_attr(test, test)]
pub fn main() {
    println!("----- binary expression example -----\n");

    let input = "1+2*(6+4)/5";

    println!(" input: {}", &input);

    let result = Expr::parse(input);

    let expr = match result {
        Done(expr, rest) => {
            println!("  rest: {}", rest);
            expr
        }
        Fail(_, rest) => {
            println!("error; rest: {}", rest.unwrap());
            return;
        }
        Fatal(e) => e.never(),
    };

    println!("result: {} = {}", display(&expr), eval(&expr));

    println!()
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
        Expr::Term(atom) => match atom {
            Term::Parenthesized(e) => display(e),
            Term::Integer(int) => format!("{}", int.0),
        },
    }
}

fn eval(expr: &Expr) -> usize {
    match expr {
        Expr::Bin(l, op, r) => {
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
            Term::Integer(n) => n.0,
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

impl Op {
    fn parse<S: Stream<Segment = str>>(input: S) -> ParseResult<S, Op, ()> {
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
                    return Done(op, input.advance(1));
                }
                _ => break 'scope,
            }
        }

        drop(chars);
        Fail((), input.into())
    }
}

impl Operator for Op {
    fn precedence(&self) -> usize {
        match self {
            Op::Add => 0,
            Op::Sub => 0,
            Op::Mul => 1,
            Op::Div => 1,
        }
    }

    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
}

#[derive(Debug)]
enum Expr {
    Bin(Box<Expr>, Op, Box<Expr>),
    Term(Term),
}

impl From<Term> for Expr {
    fn from(args: Term) -> Self {
        Expr::Term(args)
    }
}

impl From<(Expr, Op, Expr)> for Expr {
    fn from((lhs, op, rhs): (Expr, Op, Expr)) -> Self {
        Expr::Bin(Box::new(lhs), op, Box::new(rhs))
    }
}

impl Expr {
    fn parse<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
        BinaryExprParser::new(Term::parse, Op::parse)
            .map(|(e, _)| e)
            .never_fault()
            .parse(input)
    }
}

#[derive(Debug)]
enum Term {
    Parenthesized(Box<Expr>),
    Integer(Integer),
}

impl Term {
    fn parse<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Term, ()> {
        Integer::parse
            .map(Term::Integer)
            .or(atom_char('(')
                .join(Expr::parse)
                .join(atom_char(')'))
                .map(|((_, e), _)| Term::Parenthesized(Box::new(e))))
            .unify()
            .map_err(|_| ())
            .never_fault()
            .parse(input)
    }
}

#[derive(Debug)]
struct Integer(usize);

impl Integer {
    fn parse<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Self, ()> {
        let chars = input.segments().flat_map(|e| e.chars());
        let radix = 10;

        let (max_digit, to_consume) = {
            let mut chars = chars.take_while(|c| c.is_digit(radix));
            if chars.next().is_none() {
                drop(chars);
                return Fail((), input.into());
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

        Done(Integer(sum), input.advance(to_consume))
    }
}
