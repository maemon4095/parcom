use std::marker::PhantomData;

use parcom::foreign::parser;
use parcom::standard::ParserExtension;
use parcom::*;

fn main() {
    let atom = parser::str::atom("a");
    let rep = atom.repeat(..4);
    let source = foreign::stream::StrStream::new("aaaaaabbbccc");

    match rep.parse(source) {
        Ok((v, r)) => {
            println!("{:?} {:?}", v, r.location(0));
        }
        Err((e, r)) => {
            println!("{:?} {:?}", e, r.location(0));
        }
    }
}

// expr = expr op expr / term
// term = atom / (expr)

enum Expr<T, O> {
    BinOp(Box<Expr<T, O>>, O, Box<Expr<T, O>>),
    Atom(T),
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

    fn parse(&self, input: S) -> ParseResult<S, Self> {
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

    fn parse_impl(&self, input: S, precedence: usize) -> ::parcom::ParseResult<S, Self> {
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

    fn parse(input: S) -> Result<(Self, S), (Self::Error, S)> {
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
