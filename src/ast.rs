use llvm_sys::prelude::LLVMValueRef;

use crate::compile::Compiler;
use crate::error::CompileError;
use crate::Result;
use crate::{error::ParserError, lex::Token};

pub trait Codegen {
    fn codegen(&self, compiler: &mut Compiler) -> Result<LLVMValueRef>;
}

#[derive(Debug)]
pub enum ExprAst {
    Number(f64),
    Variable(String),
    Binary(Box<BinaryExprAst>),
    Call(Box<CallExprAst>),
}

impl Codegen for ExprAst {
    fn codegen(&self, compiler: &mut Compiler) -> Result<LLVMValueRef> {
        match self {
            ExprAst::Number(f) => Ok(compiler.const_double(*f)),
            ExprAst::Variable(name) => match compiler.variable(name) {
                Some(val) => Ok(val),
                None => Err(CompileError::UnknowVariableName(name.clone()).into()),
            },
            ExprAst::Binary(binary_expr_ast) => binary_expr_ast.codegen(compiler),
            ExprAst::Call(call_expr_ast) => call_expr_ast.codegen(compiler),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpSymbol {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    Greater,
}

impl TryFrom<Token> for OpSymbol {
    type Error = ParserError;

    fn try_from(value: Token) -> std::result::Result<Self, Self::Error> {
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

impl Codegen for BinaryExprAst {
    fn codegen(&self, compiler: &mut Compiler) -> Result<LLVMValueRef> {
        let left = self.lhs.codegen(compiler)?;
        let right = self.rhs.codegen(compiler)?;

        let name = match self.op {
            OpSymbol::Add => "addtmp",
            OpSymbol::Sub => "subtmp",
            OpSymbol::Mul => "multmp",
            OpSymbol::Div => "divtmp",
            OpSymbol::Less | OpSymbol::Greater => "cmptmp",
        };

        Ok(compiler.create_binary(left, right, name, self.op))
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

impl Codegen for CallExprAst {
    fn codegen(&self, compiler: &mut Compiler) -> Result<LLVMValueRef> {
        compiler.create_call(self.call.as_str(), &self.args, "calltmp")
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
