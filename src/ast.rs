#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Bool(BoolLiteral),
    Block { expr: Vec<BlockElement> },
    DecimalInt(u64),
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolLiteral {
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct BinOp<T> {
    pub left: Box<Expr>,
    pub op: T,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AddSub {
    pub left: Box<Expr>,
    pub op: AddSubOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum AddSubOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum MulDivOp {
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum BlockElement {
    Expr(Expr),
    Var { name: String, def: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub child: Option<Box<Identifier>>,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FunctionApplication {
    pub fident: Identifier,
    pub options: std::collections::BTreeMap<String, PrimaryExpr>,
    pub args: Vec<PrimaryExpr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    AddSub(BinOp<AddSubOp>),
    MulDiv(BinOp<MulDivOp>),
    Primary(PrimaryExpr),
    FunctionApplication(FunctionApplication),
}
