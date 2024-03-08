use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, u64},
    combinator::{eof, map, opt},
    error::Error,
    multi::separated_list0,
    number::complete::be_u64,
    sequence::{delimited, pair},
    IResult,
};

#[derive(Debug, Clone)]
pub enum ReplAst {
    Epsilon,
    Bool(BoolLiteral),
    Block { expr: Vec<ReplAst> },
    DecimalInt(u64),
}

#[derive(Debug, Clone)]
pub enum BoolLiteral {
    True,
    False,
}

fn pbool(input: &str) -> IResult<&str, BoolLiteral> {
    let pf = map(tag("false"), |_| BoolLiteral::False);
    let pt = map(tag("true"), |_| BoolLiteral::True);
    alt((pf, pt))(input)
}

pub fn parse_line(input: &str) -> IResult<&str, ReplAst> {
    let pb = map(pbool, |b| ReplAst::Bool(b));
    let none = map(eof, |_| ReplAst::Epsilon);
    let block = map(block, |b| ReplAst::Block { expr: b });
    let u = map(u64, |u| ReplAst::DecimalInt(u));
    alt((pb, none, block, u))(input)
}

fn block(input: &str) -> IResult<&str, Vec<ReplAst>> {
    let inner = separated_list0(char(';'), parse_line);
    delimited(char('{'), inner, char('}'))(input)
}
