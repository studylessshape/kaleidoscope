use crate::{error::ParserError, lex::Token};

#[derive(Debug)]
pub enum ExprAst {
    Number(f64),
    Variable(String),
    Binary(Box<BinaryExprAst>),
    Call(Box<CallExprAst>),
}

#[derive(Debug)]
pub enum OpSymbol {
    Add,
    Sub,
    Mul,
    Div,
}

impl TryFrom<Token> for OpSymbol {
    type Error = ParserError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Add => Ok(Self::Add),
            Token::Minus => Ok(Self::Sub),
            Token::Mul => Ok(Self::Mul),
            Token::Div => Ok(Self::Div),
            _ => Err(ParserError::ParseOpSymbolError(value)),
        }
    }
}

#[derive(Debug)]
pub struct BinaryExprAst {
    pub op: OpSymbol,
    pub lhs: ExprAst,
    pub rhs: ExprAst,
}

impl BinaryExprAst {
    pub fn new(op: OpSymbol, lhs: ExprAst, rhs: ExprAst) -> Self {
        Self { op, lhs, rhs }
    }
}

#[derive(Debug)]
pub struct CallExprAst {
    pub call: String,
    pub args: Vec<ExprAst>,
}

impl CallExprAst {
    pub fn new(call: String, args: Vec<ExprAst>) -> Self {
        Self { call, args }
    }
}

#[derive(Debug)]
pub struct PrototypeAst {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct FunctionAst {
    pub proto: PrototypeAst,
    pub body: ExprAst,
}
