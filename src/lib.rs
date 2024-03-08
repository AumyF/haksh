use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{eof, map},
    IResult,
};

#[derive(Debug, Clone)]
pub enum ReplAst {
    Epsilon,
    Bool(BoolLiteral),
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
    alt((pb, none))(input)
}
