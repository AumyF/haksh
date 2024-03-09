#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Bool(BoolLiteral),
    Block { expr: Vec<Expr> },
    DecimalInt(u64),
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
pub enum Expr {
    AddSub(BinOp<AddSubOp>),
    Primary(PrimaryExpr),
}
