use std::fmt::Debug;
use std::marker::PhantomData;

use parcom::foreign::parser::str::atom;
use parcom::standard::ParseExtension;
use parcom::standard::ParserExtension;
use parcom::Parser;
use parcom::*;

fn main() {
    let input = "10*(10+3)/4+12";

    let result = expr(input);

    let expr = match result {
        Ok((expr, _)) => expr,
        Err(_) => return,
    };

    println!("{}", eval(&expr));
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

fn expr<S: RewindStream<Segment = str>>(input: S) -> ParseResult<S, Expr<Term<Op>, Op>, ()> {
    let result = ExprParser::new(term, Op::into_parser()).parse(input);
    result
}

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

// expr = expr op expr / term
// term = atom / (expr)
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

enum Associativity {
    Left,
    Right,
}

trait Operator {
    fn precedence(&self) -> usize;
    fn associativity(&self) -> Associativity;
}

struct ExprParser<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>>
where
    POp::Output: Operator,
{
    parser_term: PTerm,
    parser_op: POp,
    marker: PhantomData<S>,
}

impl<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>> Parser<S> for ExprParser<S, PTerm, POp>
where
    POp::Output: Operator,
{
    type Output = Expr<PTerm::Output, POp::Output>;
    type Error = PTerm::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        self.parse_impl(input, 0)
    }
}

impl<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>> ExprParser<S, PTerm, POp>
where
    POp::Output: Operator,
{
    fn new(parser_term: PTerm, parser_op: POp) -> Self {
        Self {
            parser_term,
            parser_op,
            marker: PhantomData,
        }
    }

    fn parse_impl(
        &self,
        input: S,
        precedence: usize,
    ) -> ::parcom::ParseResult<S, Expr<PTerm::Output, POp::Output>, PTerm::Error> {
        let (term, mut rest) = match self.parser_term.parse(input) {
            Ok(t) => t,
            Err((e, r)) => return Err((e, r)),
        };

        let mut operand = Expr::Atom(term);
        let mut last_anchor = rest.anchor();

        loop {
            let (op, r) = match self.parser_op.parse(rest) {
                Ok((op, r)) if op.precedence() >= precedence => (op, r),
                Ok((_, r)) => {
                    rest = r.rewind(last_anchor);
                    break;
                }
                Err((_, r)) => {
                    rest = r.rewind(last_anchor);
                    break;
                }
            };

            let next_prec = match op.associativity() {
                Associativity::Left => precedence + 1,
                Associativity::Right => precedence,
            };

            let (rhs, r) = self.parse_impl(r, next_prec)?;

            operand = Expr::BinOp(Box::new(operand), op, Box::new(rhs));
            last_anchor = r.anchor();
            rest = r;
        }

        Ok((operand, rest))
    }
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

impl<S: Stream<Segment = str>> Parse<S> for Op {
    type Error = ();

    fn parse(input: S) -> ParseResult<S, Self, Self::Error> {
        let mut chars = input.segments().flat_map(|e| e.chars());

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
