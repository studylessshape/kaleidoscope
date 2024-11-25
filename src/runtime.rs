use std::{cell::RefCell, collections::HashMap, ffi::CString};

use llvm_sys::{
    core::{
        LLVMBuildCall2, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFSub, LLVMBuildUIToFP, LLVMConstReal, LLVMContextCreate, LLVMCountParams, LLVMCreateBuilderInContext, LLVMCreateFunctionPassManagerForModule, LLVMDoubleType, LLVMDoubleTypeInContext, LLVMGetNamedFunction, LLVMGlobalGetValueType, LLVMModuleCreateWithNameInContext
    }, prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMValueRef}, LLVMBuilder, LLVMCallConv, LLVMRealPredicate, LLVMValue
};
use crate::{ast::{Codegen, ExprAst, OpSymbol}, error::ParserError, Result};

pub struct RuntimeBuilder {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    names: HashMap<String, LLVMValueRef>,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let name =  CString::new("my tool jit").unwrap();
            Self {
                builder: LLVMCreateBuilderInContext(context),
                module: LLVMModuleCreateWithNameInContext(name.as_ptr(), context),
                context,
                names: HashMap::new(),
            }
        }
    }

    pub fn const_double(&self, val: f64) -> LLVMValueRef {
        unsafe {
            let double_type = LLVMDoubleTypeInContext(self.context);
            let value = LLVMConstReal(double_type, val);
            value
        }
    }

    pub fn variable<S>(&self, name: S) -> Option<LLVMValueRef>
    where
        S: AsRef<[u8]>,
    {
        self.names
            .get(&String::from_utf8_lossy(name.as_ref()).to_string())
            .copied()
    }

    pub fn create_binary(&self, left: LLVMValueRef, right: LLVMValueRef, name: &str, op: OpSymbol) -> LLVMValueRef {
        unsafe {
            let name = CString::new(name).unwrap();

            match op {
                OpSymbol::Add => LLVMBuildFAdd(self.builder, left, right, name.as_ptr()),
                OpSymbol::Sub => LLVMBuildFSub(self.builder, left, right, name.as_ptr()),
                OpSymbol::Mul => LLVMBuildFMul(self.builder, left, right, name.as_ptr()),
                OpSymbol::Div => LLVMBuildFDiv(self.builder, left, right, name.as_ptr()),
                OpSymbol::Less | OpSymbol::Greater => {
                    let cmp = if let OpSymbol::Less = op {
                        LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealULT, left, right, name.as_ptr())
                    } else {
                        LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealULT, right, left, name.as_ptr())
                    };
                    let bool_name = CString::new("booltmp").unwrap();
                    LLVMBuildUIToFP(self.builder, cmp, LLVMDoubleTypeInContext(self.context), bool_name.as_ptr())
                }
            }
        }
    }

    pub fn create_call(&mut self, call: &str, args: &Vec<ExprAst>, name: &str) -> Result<LLVMValueRef> {
        unsafe {
            let call = CString::new(call).unwrap();
            let function = LLVMGetNamedFunction(self.module, call.as_ptr());
            if function.is_null() {
                return Err(ParserError::UnknowFunction(call.to_string_lossy().to_string()).into());
            }

            let args_size = LLVMCountParams(function);
            if args_size as usize != args.len() {
                return Err(ParserError::IncorrectArguments(args_size as usize, args.len()).into());
            }

            let mut args_val = Vec::new();
            let rc = RefCell::new(self);
            for arg in args {
                let arg_v = arg.codegen(&mut rc.borrow_mut())?;
                if arg_v.is_null() {
                    return Err(ParserError::FunctionArgumentsError.into());
                }
                args_val.push(arg_v);
            }
            
            let name = CString::new(name).unwrap();
            let builder = rc.borrow().builder;
            Ok(LLVMBuildCall2(builder, LLVMGlobalGetValueType(function), function, args_val.as_mut_ptr(), args_size, name.as_ptr()))
        }
    }
}

pub struct Env {
    names: HashMap<String, *mut LLVMValue>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }
}
