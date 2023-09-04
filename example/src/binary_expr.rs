#![cfg_attr(test, cfg(test))]

use parcom::foreign::parser::str::atom;
use parcom::standard::binary_expr::*;
use parcom::standard::ParserExtension;
use parcom::Parser;
use parcom::*;

/// parsing binary expression example. parse and eval expression with syntax below
/// expr = expr op expr / term
/// term = integer / (expr)
#[cfg_attr(test, test)]
pub fn main() {
    println!("----- binary expression example -----\n");

    let input = "((((10 * (10 + 6) / 4 + 12))))u8";

    println!("input: {}", input);

    let result = expr(input);

    let expr = match result {
        Ok((expr, rest)) => {
            println!("rest of input: {}", rest);
            expr
        }
        Err(_) => {
            println!("err");
            return;
        }
    };

    let evaluated = eval(&expr);
    println!("evaluated: {v}", v = evaluated);

    println!();
}

/// expr = expr op expr / term
fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr<Term<Op>, Op>, ()> {
    BinaryExprParser::new(
        term.map(|t| Expr::Atom(t)),
        space.join(op).join(space).map(|((_, op), _)| op),
    )
    .parse(input)
}

/// term = integer / (expr)
fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Term<Op>, ()> {
    integer
        .or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
        .map(|e| match e {
            standard::Either::First(n) => Term::Integer(n),
            standard::Either::Last(e) => Term::Parenthesized(Box::new(e)),
        })
        .map_err(|_| ())
        .parse(input)
}

fn eval(expr: &Expr<Term<Op>, Op>) -> usize {
    match expr {
        Expr::BinOp(l, op, r) => match op {
            Op::Add => eval(l) + eval(r),
            Op::Sub => eval(l) - eval(r),
            Op::Mul => eval(l) * eval(r),
            Op::Div => eval(l) / eval(r),
        },
        Expr::Atom(atom) => match atom {
            Term::Parenthesized(e) => eval(e),
            Term::Integer(n) => *n,
        },
    }
}

#[derive(Debug)]
enum Expr<T, O> {
    BinOp(Box<Expr<T, O>>, O, Box<Expr<T, O>>),
    Atom(T),
}

#[derive(Debug)]
enum Term<O> {
    Parenthesized(Box<Expr<Term<O>, O>>),
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
    type Expr = Expr<Term<Self>, Self>;
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

    fn construct(self, lhs: Self::Expr, rhs: Self::Expr) -> Self::Expr {
        Expr::BinOp(Box::new(lhs), self, Box::new(rhs))
    }
}

fn space<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, (), ()> {
    foreign::parser::str::atom_char(' ')
        .map(|_| ())
        .repeat(1..)
        .map(|_| ())
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
        _ => return Err(((), input)),
    };

    Ok((op, input.advance(1)))
}

fn integer<S: Stream<Segment = str>>(input: S) -> ParseResult<S, usize, ()> {
    let chars = input.segments().flat_map(|e| e.chars());
    let radix = 10;

    let (max_digit, to_consume) = {
        let mut chars = chars.take_while(|c| c.is_digit(radix));
        if chars.next().is_none() {
            drop(chars);
            return Err(((), input));
        }

        let mut digit = 1;
        let mut consume = 1;

        for _ in chars {
            digit *= 10;
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

        sum += d as usize * digit;
        digit /= 10;
    }

    Ok((sum, input.advance(to_consume)))
}
