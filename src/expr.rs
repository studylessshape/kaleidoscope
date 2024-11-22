pub enum Expr {
    Number(f64),
    Variable(String),
    Binary(Box<BinaryExpr>),
    Call(Box<CallExpr>)
}

pub enum OperateSymbol {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct BinaryExpr {
    op: OperateSymbol,
    lhs: Expr,
    rhs: Expr,
}

pub struct CallExpr {
    call: String,
    args: Vec<Expr>,
}