use crate::ast::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Properties(BTreeMap<String, Value>);

#[derive(Debug, Clone)]
pub struct Environment(BTreeMap<String, Value>);

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

impl Environment {
    pub fn new() -> Environment {
        Environment(BTreeMap::new())
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }
    pub fn set(&self, name: &str, value: Value) -> Environment {
        let mut new = self.clone();
        new.0.insert(name.to_string(), value);
        new
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Compound { properties: Properties },
    UInt64(u64),
    Bool(bool),
    Unit,
    String(String),
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
    fn evaluate(&self, env: &Environment) -> EvalResult {
        match self {
            PrimaryExpr::Bool(b) => Ok(b.evaluate()),
            PrimaryExpr::Block { expr } => Ok(expr
                .iter()
                .try_fold((env.clone(), Value::Unit), |(env, _), expr| {
                    expr.evaluate(&env)
                })?
                .1),
            PrimaryExpr::DecimalInt(n) => Ok(Value::UInt64(*n)),
            PrimaryExpr::Identifier(name) => env
                .get(name)
                .cloned()
                .ok_or(format!("no variable named {name} found")),
        }
    }
}

impl BlockElement {
    pub fn evaluate(&self, env: &Environment) -> Result<(Environment, Value), String> {
        match self {
            BlockElement::Expr(e) => Ok((env.clone(), e.evaluate(env)?)),
            BlockElement::Var { name, def } => {
                let env = env.set(name, def.evaluate(env)?);
                Ok((env, Value::Unit))
            }
        }
    }
}

pub trait BinaryOperator: Sized {
    fn op(&self, lhs: Value, rhs: Value) -> EvalResult;
    fn into_expr() -> impl Fn(BinOp<Self>) -> Expr;
}

impl<T: BinaryOperator> BinOp<T> {
    fn evaluate(&self, env: &Environment) -> EvalResult {
        let left = self.left.evaluate(env)?;
        let right = self.right.evaluate(env)?;
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
    fn into_expr() -> impl Fn(BinOp<AddSubOp>) -> Expr {
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

impl FunctionApplication {
    fn evaluate(&self, env: &Environment) -> EvalResult {
        match self.fident.clone() {
            i if i
                == Identifier {
                    path: "fs".to_string(),
                    child: Some(Box::new(Identifier {
                        path: "cwd".to_string(),
                        child: None,
                    })),
                } =>
            {
                let current_dir = std::env::current_dir().map_err(|e| "IOError")?;

                Ok(current_dir
                    .to_str()
                    .ok_or(format!("to_str error"))
                    .map(|s| Value::String(s.to_string()))?)
            }

            _ => Err("Not found".to_string()),
        }
    }
}

impl Expr {
    pub fn evaluate(&self, env: &Environment) -> EvalResult {
        match self {
            Expr::AddSub(e) => e.evaluate(env),
            Expr::MulDiv(e) => e.evaluate(env),
            Expr::Primary(e) => e.evaluate(env),
            Expr::FunctionApplication(e) => e.evaluate(env),
        }
    }
}
