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
    Compound {
        properties: Properties,
    },
    UInt64(u64),
    Bool(bool),
    Unit,
    String(String),
    Fn {
        env: Environment,
        body: Block,
        params: Vec<String>,
        name: Option<String>,
    },
}

impl Value {
    fn try_get_u64(&self) -> Option<u64> {
        match self {
            Value::UInt64(n) => Some(*n),
            _ => None,
        }
    }
    fn try_evaluate_as_fn(&self, arguments: Vec<Value>) -> EvalResult {
        match self {
            Value::Fn {
                env,
                body,
                params,
                name,
            } => {
                let env = if let Some(name) = name {
                    env.set(&name, self.clone())
                } else {
                    env.clone()
                };
                let env = params
                    .iter()
                    .enumerate()
                    .try_fold(env, |env, (index, name)| {
                        let arg = arguments.get(index).ok_or_else(|| {
                            format!(
                                "Expected {} arguments but got {}",
                                params.len(),
                                arguments.len()
                            )
                        })?;

                        let env = env.set(name, arg.clone());

                        Ok::<_, String>(env)
                    })?;

                body.evaluate(&env)
            }
            _ => Err("Not fn".to_string()),
        }
    }
}

type EvalResult = Result<Value, String>;

impl BoolLiteral {
    fn evaluate(&self) -> Value {
        Value::Bool(*self == BoolLiteral::True)
    }
}

impl Block {
    pub fn evaluate(&self, env: &Environment) -> EvalResult {
        fn a(
            mut expr: std::collections::VecDeque<BlockElement>,
            env: Environment,
            value: Value,
        ) -> EvalResult {
            Ok(if let Some(e) = expr.pop_front() {
                match e {
                    BlockElement::Expr(e) => {
                        let v = e.evaluate(&env)?;
                        a(expr, env, v)?
                    }
                    BlockElement::Var { name, def } => {
                        let env = env.set(&name, def.evaluate(&env)?);
                        a(expr, env, Value::Unit)?
                    }
                    BlockElement::AnonymousFunction(AnonymousFunction { params, body }) => a(
                        expr,
                        env.clone(),
                        Value::Fn {
                            env: env.clone(),
                            name: None,
                            params,
                            body,
                        },
                    )?,
                    BlockElement::Using { name, mut def } => {
                        def.args.push(PrimaryExpr::Block(Block(vec![
                            BlockElement::AnonymousFunction(AnonymousFunction {
                                params: vec![name],
                                body: Block(expr.into()),
                            }),
                        ])));

                        def.evaluate(&env)?
                    }
                }
            } else {
                value
            })
        }
        a(self.0.clone().into(), env.clone(), Value::Unit)
    }
}

impl BlockElement {
    pub fn evaluate_for_repl(&self, env: &Environment) -> Result<(Environment, Value), String> {
        match self {
            BlockElement::Expr(e) => Ok((env.clone(), e.evaluate(env)?)),
            BlockElement::Var { name, def } => {
                let env = env.set(name, def.evaluate(env)?);
                Ok((env, Value::Unit))
            }
            BlockElement::AnonymousFunction(AnonymousFunction { params, body }) => Ok((
                env.clone(),
                Value::Fn {
                    env: env.clone(),
                    name: None,
                    params: params.to_vec(),
                    body: body.clone(),
                },
            )),
            BlockElement::Using { .. } => {
                Err("'using' statement does not work in REPL".to_string())
            }
        }
    }
}

impl PrimaryExpr {
    fn evaluate(&self, env: &Environment) -> EvalResult {
        match self {
            PrimaryExpr::Bool(b) => Ok(b.evaluate()),
            PrimaryExpr::Block(b) => b.evaluate(&env),
            PrimaryExpr::DecimalInt(n) => Ok(Value::UInt64(*n)),
            PrimaryExpr::Identifier(name) => env
                .get(name)
                .cloned()
                .ok_or(format!("no variable named {name} found")),
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
            i if i
                == Identifier {
                    path: "println".to_string(),
                    child: None,
                } =>
            {
                // 遅そう
                let s = self
                    .args
                    .iter()
                    .map(|a| Ok(format!("{:?}", a.evaluate(env)?)))
                    .collect::<Result<Vec<_>, String>>()?
                    .join(" ");
                println!("{s}");

                Ok(Value::Unit)
            }
            i if i
                == Identifier {
                    path: "twice".to_string(),
                    child: None,
                } =>
            {
                let continuation = self.args.last().ok_or("no arguments".to_string())?;
                let _ = continuation
                    .evaluate(env)?
                    .try_evaluate_as_fn(vec![Value::Unit])?;
                let _ = continuation
                    .evaluate(env)?
                    .try_evaluate_as_fn(vec![Value::Unit])?;

                Ok(Value::Unit)
            }

            i if i
                == Identifier {
                    path: "http".to_string(),
                    child: Some(Box::new(Identifier {
                        path: "post".to_string(),
                        child: None,
                    })),
                } =>
            {
                let res =
                    reqwest::blocking::get("https://example.com").map_err(|e| e.to_string())?;

                let body = res.text().map_err(|e| e.to_string())?;

                Ok(Value::String(body))
            }

            i if i
                == Identifier {
                    path: "fs".to_string(),
                    child: Some(Box::new(Identifier {
                        path: "watch".to_string(),
                        child: None,
                    })),
                } =>
            {
                use notify::EventKind;
                use notify::Watcher;
                use notify_debouncer_full::DebouncedEvent;

                use std::fs::File;
                use std::io::{self, BufRead, Read, Seek, SeekFrom};

                let (tx, rx) = std::sync::mpsc::channel();

                let f = File::open("./latest.log").map_err(|e| e.to_string())?;

                let mut bufr = std::io::BufReader::new(f);
                bufr.seek(SeekFrom::End(0));

                let cont = self
                    .args
                    .last()
                    .ok_or("no arguments".to_string())?
                    .evaluate(env)?;

                let mut debouncer = notify_debouncer_full::new_debouncer(
                    std::time::Duration::from_secs(1),
                    None,
                    tx,
                )
                .map_err(|e| e.to_string())?;

                debouncer
                    .watcher()
                    .watch(
                        std::path::Path::new("./latest.log"),
                        notify::RecursiveMode::NonRecursive,
                    )
                    .map_err(|e| e.to_string())?;

                for res in rx {
                    match res {
                        Ok(events) => {
                            let includes_modify = events.iter().any(|f| match f.kind {
                                EventKind::Modify(_) => true,
                                _ => false,
                            });

                            if includes_modify {
                                let mut string = String::new();
                                bufr.read_to_string(&mut string).unwrap();
                                string.lines().for_each(|line| {
                                    // TODO error handling
                                    let _ = cont
                                        .try_evaluate_as_fn(vec![Value::String(line.to_string())])
                                        .unwrap();
                                });
                            }
                        }
                        Err(e) => eprintln!("watch error: {:?}", e),
                    }
                }

                Ok(Value::Unit)
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
