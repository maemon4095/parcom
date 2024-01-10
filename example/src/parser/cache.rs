#![cfg_attr(test, cfg(test))]

use chrono::Local;
use parcom::{
    std::{
        binary_expr::{Associativity, BinaryExprParser, Operator},
        primitive::str::{atom, atom_char},
        Either, ParserExtension,
    },
    ParseResult::{self, *},
    Parser, RewindStream, Stream,
};
use std::time::Instant;
use std::{fs::File, io::Write};
use tempfile::tempdir;

#[cfg_attr(test, test)]
pub fn main() {
    println!("----- cache example -----\n");
    println!("{:?}", std::env::current_dir());

    let mut no_cache_samples = Vec::new();
    let max_depth = 256;
    let poplation = 32;
    for depth in 0..max_depth {
        let input = {
            let mut s = String::with_capacity(depth * 2 + 1);
            s.extend(std::iter::repeat('(').take(depth));
            s.push('0');
            s.extend(std::iter::repeat(')').take(depth));
            s
        };
        let input_str = input.as_str();

        let durations: Vec<_> = std::iter::repeat_with(|| {
            let start = Instant::now();
            let _ = expr(input_str);
            let end = Instant::now();
            end.duration_since(start)
        })
        .take(poplation)
        .collect();

        let mean = durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128;
        no_cache_samples.push((depth as f64, mean as f64));

        println!("parse: {}", input);
        println!("elapsed: {}", mean);
        println!()
    }

    println!();

    let svg = crate::line_chart::draw(
        (848, 600),
        "elapsed",
        "depth",
        "ns",
        &[("no cache", &no_cache_samples)],
    );

    let dir = tempdir().unwrap();
    let name = Local::now().format("%F_%H%I%M%S.svg").to_string();
    let path = dir.path().join(name);
    let mut file = File::create(&path).unwrap();
    write!(file, "{}", svg).unwrap();
    match opener::open(path) {
        Ok(_) => {
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("summary file opened.");
        }
        Err(_) => {
            println!("cannot open summary file.")
        }
    }
}

/// expr = expr op expr / term
fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
    BinaryExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op))
        .map(|(e, _)| e)
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
    BinOp(Box<Expr>, Op, Box<Expr>),
    Term(Box<Term>),
}

impl From<Term> for Expr {
    fn from(args: Term) -> Self {
        Expr::Term(Box::new(args))
    }
}

impl From<(Expr, Op, Expr)> for Expr {
    fn from((lhs, op, rhs): (Expr, Op, Expr)) -> Self {
        Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
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
fn space<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, (), ()> {
    atom_char(' ').discard().repeat(1..).discard().parse(input)
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
        _ => return Fail((), input.into()),
    };

    Done(op, input.advance(1))
}

fn zero<S: Stream<Segment = str>>(input: S) -> ParseResult<S, char, ()> {
    atom_char('0').parse(input)
}
