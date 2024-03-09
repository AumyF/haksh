use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, u64},
    combinator::{eof, map},
    multi::{many1, separated_list0},
    sequence::{delimited, pair, terminated},
    IResult,
};

use crate::ast::*;

type Line = Expr;

pub fn parse_line(input: &str) -> IResult<&str, Line> {
    map(pair(expr, eof), |(li, _)| li)(input)
}

fn expr(input: &str) -> IResult<&str, Expr> {
    alt((map(add_sub, Expr::AddSub), map(primary_expr, Expr::Primary)))(input)
}

fn add_sub(input: &str) -> IResult<&str, AddSub> {
    let add = map(char('+'), |_| AddSubOp::Add);
    let sub = map(char('-'), |_| AddSubOp::Sub);
    let op = alt((add, sub));
    map(
        pair(
            terminated(primary_expr, space0),
            many1(pair(
                terminated(op, space0),
                terminated(primary_expr, space0),
            )),
        ),
        |(first, rest)| {
            let (op1, right1) = rest.first().cloned().unwrap();
            (&rest[1..]).into_iter().fold(
                AddSub {
                    left: Box::new(Expr::Primary(first)),
                    op: op1,
                    right: Box::new(Expr::Primary(right1)),
                },
                |acc, e| AddSub {
                    left: Box::new(Expr::AddSub(acc)),
                    op: e.0.clone(),
                    right: Box::new(Expr::Primary(e.1.clone())),
                },
            )
        },
    )(input)
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
