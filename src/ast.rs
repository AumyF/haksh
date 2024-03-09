#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Epsilon,
    Bool(BoolLiteral),
    Block { expr: Vec<PrimaryExpr> },
    DecimalInt(u64),
}

#[derive(Debug, Clone)]
pub enum BoolLiteral {
    True,
    False,
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
    AddSub(AddSub),
    Primary(PrimaryExpr),
}
