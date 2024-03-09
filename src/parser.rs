use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, u64},
    combinator::{eof, map},
    error::ParseError,
    multi::{many1, separated_list0},
    sequence::{delimited, pair, terminated},
    IResult, Parser,
};

use crate::ast::*;

type Line = Expr;

pub fn parse_line(input: &str) -> IResult<&str, Line> {
    map(pair(expr, eof), |(li, _)| li)(input)
}

fn expr(input: &str) -> IResult<&str, Expr> {
    alt((
        map(add_sub, Expr::AddSub),
        map(mul_div, Expr::MulDiv),
        map(primary_expr, Expr::Primary),
    ))(input)
}

fn add_sub(input: &str) -> IResult<&str, BinOp<AddSubOp>> {
    let add = map(char('+'), |_| AddSubOp::Add);
    let sub = map(char('-'), |_| AddSubOp::Sub);
    let op = alt((add, sub));
    let term = |input| alt((map(mul_div, Expr::MulDiv), map(primary_expr, Expr::Primary)))(input);
    binop(term, op)(input)
}

fn binop<'t, Op: crate::interpreter::BinaryOperator, E: ParseError<&'t str>, G>(
    term: fn(&'t str) -> IResult<&'t str, Expr, E>,
    op: G,
) -> impl FnOnce(&'t str) -> IResult<&'t str, BinOp<Op>, E>
where
    G: Parser<&'t str, Op, E>,
{
    let op = terminated(op, space0);
    move |input: &str| {
        map(
            pair(
                terminated(term, space0),
                many1(pair(op, terminated(term, space0))),
            ),
            |(first, rest)| {
                let mut rest: std::collections::VecDeque<_> = rest.into_iter().collect();
                let (op1, right1) = rest.pop_front().unwrap();

                rest.into_iter().fold(
                    BinOp {
                        left: Box::new(first),
                        op: op1,
                        right: Box::new(right1),
                    },
                    |acc, (op, e)| BinOp {
                        left: Box::new(Op::into_expr()(acc)),
                        op,
                        right: Box::new(e),
                    },
                )
            },
        )(input)
    }
}

fn mul_div(input: &str) -> IResult<&str, BinOp<MulDivOp>> {
    let term = |input| map(primary_expr, Expr::Primary)(input);

    let add = map(char('*'), |_| MulDivOp::Mul);
    let sub = map(char('/'), |_| MulDivOp::Div);
    let op = alt((add, sub));
    binop(term, op)(input)
}

fn pbool(input: &str) -> IResult<&str, BoolLiteral> {
    let pf = map(tag("false"), |_| BoolLiteral::False);
    let pt = map(tag("true"), |_| BoolLiteral::True);
    alt((pf, pt))(input)
}

pub fn primary_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    let pb = map(pbool, |b| PrimaryExpr::Bool(b));
    let block = map(block, |b| PrimaryExpr::Block { expr: b });
    let u = map(u64, |u| PrimaryExpr::DecimalInt(u));
    alt((pb, block, u))(input)
}

fn block(input: &str) -> IResult<&str, Vec<Expr>> {
    let inner = separated_list0(char(';'), expr);
    delimited(char('{'), inner, char('}'))(input)
}
