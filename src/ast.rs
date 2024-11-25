use std::ptr::null;

use llvm_sys::core::{
    LLVMBFloatTypeInContext, LLVMConstReal, LLVMContextCreate, LLVMCreateBuilder,
    LLVMCreateBuilderInContext, LLVMDoubleType,
};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfFloat;
use llvm_sys::prelude::{LLVMContextRef, LLVMValueRef};
use llvm_sys::LLVMValue;

use crate::runtime::RuntimeBuilder;
use crate::Result;
use crate::{error::ParserError, lex::Token};

pub trait Codegen {
    fn codegen(&self, builder: &mut RuntimeBuilder) -> Result<LLVMValueRef>;
}

#[derive(Debug)]
pub enum ExprAst {
    Number(f64),
    Variable(String),
    Binary(Box<BinaryExprAst>),
    Call(Box<CallExprAst>),
}

impl Codegen for ExprAst {
    fn codegen(&self, builder: &mut RuntimeBuilder) -> Result<LLVMValueRef> {
        match self {
            ExprAst::Number(f) => Ok(builder.const_double(*f)),
            ExprAst::Variable(name) => match builder.variable(name) {
                Some(val) => Ok(val),
                None => Err(ParserError::UnknowVariableName(name.clone()).into()),
            },
            ExprAst::Binary(binary_expr_ast) => todo!(),
            ExprAst::Call(call_expr_ast) => todo!(),
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
    fn codegen(&self, builder: &mut RuntimeBuilder) -> Result<LLVMValueRef> {
        let left = self.lhs.codegen(builder)?;
        let right = self.rhs.codegen(builder)?;

        let name = match self.op {
            OpSymbol::Add => "addtmp",
            OpSymbol::Sub => "subtmp",
            OpSymbol::Mul => "multmp",
            OpSymbol::Div => "divtmp",
            OpSymbol::Less | OpSymbol::Greater => "cmptmp",
        };

        Ok(builder.create_binary(left, right, name, self.op))
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
    fn codegen(&self, builder: &mut RuntimeBuilder) -> Result<LLVMValueRef> {
        todo!()
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
