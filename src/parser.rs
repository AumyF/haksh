use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, line_ending, multispace0, space0, space1, u64},
    combinator::{eof, map},
    error::ParseError,
    multi::{many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use nom_regex::str::re_find;

use crate::ast::*;
use std::collections::BTreeMap;

type Line = BlockElement;

pub fn parse_line(input: &str) -> IResult<&str, Line> {
    map(pair(block_element, eof), |(li, _)| li)(input)
}

fn expr(input: &str) -> IResult<&str, Expr> {
    let pif = tuple((tag("if"),space0, expr,space0, tag("then"),space0, block, space0,tag("else"), space0,block));

    alt((
        map(pif, |(_if, _,cond, _,_then,_, true_exp, _,_else,_, false_expr)| {
            Expr::If(If {
                cond: Box::new(cond),
                true_exp: Box::new(Expr::Primary(PrimaryExpr::Block(Block(true_exp)))),
                false_expr: Box::new(Expr::Primary(PrimaryExpr::Block(Block(false_expr)))),
            })
        }),
        map(function_application, Expr::FunctionApplication),
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

fn space<'t, O, E: ParseError<&'t str>>(
    p: fn(&'t str) -> IResult<&'t str, O, E>,
) -> impl Fn(&'t str) -> IResult<&'t str, O, E> {
    move |input| delimited(space0, p, space0)(input)
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

fn function_name(input: &str) -> IResult<&str, Vec<String>> {
    separated_list1(char('.'), identifer)(input)
}

fn function_application(input: &str) -> IResult<&str, FunctionApplication> {
    enum Type {
        Option(String, PrimaryExpr),
        Arg(PrimaryExpr),
    }
    let identifier = map(space(function_name), |i| {
        let mut i = i.iter().rev();
        let a = i.next().unwrap();
        i.fold(
            Identifier {
                child: None,
                path: a.to_string(),
            },
            |acc, path| Identifier {
                child: Some(Box::new(acc)),
                path: path.to_string(),
            },
        )
    });

    // let option = preceded(tag("--"), pair(space(identifer), space(primary_expr)));
    // let option = map(option, |(k, v)| Type::Option(k, v));
    let arg = map(primary_expr, Type::Arg);
    let opargs = separated_list0(space1, arg);

    let r = tuple((identifier, opargs));
    map(r, |(fident, opargs)| {
        let mut args = Vec::new();
        let mut options = BTreeMap::new();
        for e in opargs {
            match e {
                Type::Arg(a) => args.push(a),
                Type::Option(k, v) => {
                    options.insert(k, v);
                }
            }
        }

        FunctionApplication {
            args,
            options,
            fident,
        }
    })(input)
}

#[test]
fn test_fa() {
    let (i, _) = function_application(
        r#"fs.cwd;
fs.cwd"#,
    )
    .unwrap();
    assert_eq!(
        i,
        r#";
fs.cwd"#
    )
}
#[test]
fn test_fab() {
    let (i, _) = function_application(r#"fs.cwd;fs.cwd"#).unwrap();
    assert_eq!(i, r#";fs.cwd"#)
}
fn pbool(input: &str) -> IResult<&str, BoolLiteral> {
    let pf = map(tag("false"), |_| BoolLiteral::False);
    let pt = map(tag("true"), |_| BoolLiteral::True);
    alt((pf, pt))(input)
}

fn pstring(input: &str) -> IResult<&str, String> {
    map(
        delimited(char('"'), take_until(r#"""#), char('"')),
        |s: &str| s.to_string(),
    )(input)
}

fn pcompound(input: &str) -> IResult<&str, std::collections::BTreeMap<String, Expr>> {
    let p = delimited(
        char('('),
        separated_list0(char(','), tuple((identifer, char('='), expr))),
        char(')'),
    );
    map(p, |e| {
        let mut map = std::collections::BTreeMap::new();
        e.iter().for_each(|(key, _, def)| {
            map.insert(key.clone(), def.clone());
        });

        map
    })(input)
}

pub fn primary_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    let pb = map(pbool, |b| PrimaryExpr::Bool(b));
    let block = map(block, |b| PrimaryExpr::Block(Block(b)));
    let u = map(u64, |u| PrimaryExpr::DecimalInt(u));
    let id = map(identifer, |id| PrimaryExpr::Identifier(id));
    let ps = map(pstring, PrimaryExpr::StringLiteral);
    let pc = map(pcompound, PrimaryExpr::Compound);
    alt((pb, block, u, id, ps, pc))(input)
}

fn identifer(input: &str) -> IResult<&str, String> {
    let re = regex::Regex::new(r"^\p{XID_Start}\p{XID_Continue}*").unwrap();
    let ident = re_find(re);

    ident(input).map(|(s, i)| (s, i.to_string()))
}

#[test]
fn test_i() {
    let (i, _) = identifer(
        r#"fs;
fs.cwd"#,
    )
    .unwrap();
    assert_eq!(
        i,
        r#";
fs.cwd"#
    );

    let mut s = separated_list1(char('.'), identifer);

    let (i, _) = s("fs.cwd;hoge").unwrap();
    assert_eq!(i, ";hoge");
}

fn block_element_using(input: &str) -> IResult<&str, BlockElement> {
    let a = tuple((
        tag("using"),
        space0,
        identifer,
        space0,
        tag("="),
        space0,
        function_application,
    ));
    map(a, |(_let, _, ident, _, _eq, _, def)| BlockElement::Using {
        name: ident.to_string(),
        def,
    })(input)
}

fn block_element(input: &str) -> IResult<&str, BlockElement> {
    let block_element_var = map(
        tuple((
            tag("let"),
            space0,
            identifer,
            space0,
            tag("="),
            space0,
            expr,
        )),
        |(_let, _, ident, _, _eq, _, def)| BlockElement::Var {
            name: ident.to_string(),
            def,
        },
    );
    let expr = map(expr, BlockElement::Expr);
    let mut block_element = alt((block_element_var, block_element_using, expr));

    block_element(input)
}

fn block_inner(input: &str) -> IResult<&str, Vec<BlockElement>> {
    separated_list0(alt((char(';'), char('\n'))), block_element)(input)
}

pub fn parse_file(input: &str) -> IResult<&str, Block> {
    let a = terminated(block_inner, pair(multispace0, eof));
    map(a, |v| Block(v))(input)
}

fn block(input: &str) -> IResult<&str, Vec<BlockElement>> {
    delimited(char('{'), block_inner, char('}'))(input)
}
