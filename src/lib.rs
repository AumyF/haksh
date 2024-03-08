use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{eof, map, opt},
    multi::separated_list0,
    sequence::{delimited, pair},
    IResult,
};

#[derive(Debug, Clone)]
pub enum ReplAst {
    Epsilon,
    Bool(BoolLiteral),
    Block { expr: Vec<ReplAst> },
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
    alt((pb, none, block))(input)
}

fn block(input: &str) -> IResult<&str, Vec<ReplAst>> {
    let inner = separated_list0(char(';'), parse_line);
    delimited(char('{'), inner, char('}'))(input)
}
