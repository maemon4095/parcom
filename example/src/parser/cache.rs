#![cfg_attr(test, cfg(test))]

use chrono::Local;
use parcom::{
    foreign::{self, parser::str::atom},
    standard::{self, binary_expr::Operator, parser::binary_expr::BinaryExprParser},
    standard::{binary_expr::Associativity, ParserExtension},
    ParseResult, Parser, RewindStream, Stream,
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
            println!("summary file opened. press enter key to exit.");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        Err(_) => {
            println!("cannot open summary file.")
        }
    }
}

/// expr = expr op expr / term
fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
    BinaryExprParser::new(term, space.join(op).join(space).map(|((_, op), _)| op)).parse(input)
}

/// term = 0 / (expr)
fn term<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr, ()> {
    zero.or(atom("(").join(expr).join(atom(")")).map(|((_, e), _)| e))
        .map(|e| match e {
            standard::Either::First(c) => Expr::Atom(c),
            standard::Either::Last(e) => Expr::Parenthesized(Box::new(e)),
        })
        .map_err(|_| ())
        .parse(input)
}

#[derive(Debug, Clone)]
enum Expr {
    BinOp(Box<Expr>, Op, Box<Expr>),
    Atom(char),
    Parenthesized(Box<Expr>),
}

#[derive(Debug, Clone)]
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
        .discard()
        .repeat(1..)
        .discard()
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

fn zero<S: Stream<Segment = str>>(input: S) -> ParseResult<S, char, ()> {
    foreign::parser::str::atom_char('0').parse(input)
}
