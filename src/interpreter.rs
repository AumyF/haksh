use crate::ast::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Properties(BTreeMap<String, Value>);

impl Properties {
    pub fn new() -> Properties {
        Properties(BTreeMap::new())
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }
    pub fn set(&mut self, name: &str, value: Value) {
        self.0.insert(name.to_string(), value);
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Compound { properties: Properties },
    UInt64(u64),
    Bool(bool),
    Unit,
}

impl Value {
    fn try_get_u64(&self) -> Option<u64> {
        match self {
            Value::UInt64(n) => Some(*n),
            _ => None,
        }
    }
}

type EvalResult = Result<Value, String>;

impl BoolLiteral {
    fn evaluate(&self) -> Value {
        Value::Bool(*self == BoolLiteral::True)
    }
}

impl PrimaryExpr {
    fn evaluate(&self) -> EvalResult {
        match self {
            PrimaryExpr::Bool(b) => Ok(b.evaluate()),
            PrimaryExpr::Block { expr } => {
                expr.iter().fold(Ok(Value::Unit), |_, expr| expr.evaluate())
            }
            PrimaryExpr::DecimalInt(n) => Ok(Value::UInt64(*n)),
        }
    }
}

pub trait BinaryOperator: Sized {
    fn op(&self, lhs: Value, rhs: Value) -> EvalResult;
    fn into_expr() -> impl Fn(BinOp<Self>) -> Expr;
}

impl<T: BinaryOperator> BinOp<T> {
    fn evaluate(&self) -> EvalResult {
        let left = self.left.evaluate()?;
        let right = self.right.evaluate()?;
        self.op.op(left, right)
    }
}

impl BinaryOperator for AddSubOp {
    fn op(&self, left: Value, right: Value) -> EvalResult {
        let left = left.try_get_u64().ok_or(format!("not int: {:?}", left))?;
        let right = right.try_get_u64().ok_or(format!("not int: {:?}", right))?;
        let result = match self {
            Self::Add => left + right,
            Self::Sub => left - right,
        };
        Ok(Value::UInt64(result))
    }
    fn into_expr() -> impl Fn(BinOp<AddSubOp>) -> Expr{
        Expr::AddSub
    }
}

impl BinaryOperator for MulDivOp {
    fn op(&self, left: Value, right: Value) -> EvalResult {
        let left = left.try_get_u64().ok_or(format!("not int: {:?}", left))?;
        let right = right.try_get_u64().ok_or(format!("not int: {:?}", right))?;
        let result = match self {
            Self::Mul => left * right,
            Self::Div => left
                .checked_div(right)
                .ok_or("division by zero".to_string())?,
        };

        Ok(Value::UInt64(result))
    }
    fn into_expr() -> impl Fn(BinOp<Self>) -> Expr {
        Expr::MulDiv
    }
}


impl Expr {
    pub fn evaluate(&self) -> EvalResult {
        match self {
            Expr::AddSub(e) => e.evaluate(),
            Expr::MulDiv(e) => e.evaluate(),
            Expr::Primary(e) => e.evaluate(),
        }
    }
}
