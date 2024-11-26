use std::{cell::RefCell, collections::HashMap, ffi::CString, mem::forget};

use crate::{
    ast::{Codegen, ExprAst, FunctionAst, OpSymbol},
    error::CompileError,
    Result,
};
use llvm_sys::{
    analysis::LLVMVerifyFunction,
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildCall2, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFSub, LLVMBuildRet, LLVMBuildUIToFP, LLVMConstReal, LLVMContextCreate, LLVMCountBasicBlocks, LLVMCountParams, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMEraseGlobalIFunc, LLVMFunctionType, LLVMGetNamedFunction, LLVMGetParams, LLVMGlobalGetValueType, LLVMModuleCreateWithNameInContext, LLVMPositionBuilderAtEnd, LLVMPrintModuleToString, LLVMPrintValueToString, LLVMSetValueName2
    },
    prelude::*,
    LLVMRealPredicate,
};

pub struct Compiler {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    names: HashMap<String, LLVMValueRef>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let name = CString::new("my tool jit").unwrap();
            Self {
                builder: LLVMCreateBuilderInContext(context),
                module: LLVMModuleCreateWithNameInContext(name.as_ptr(), context),
                context,
                names: HashMap::new(),
            }
        }
    }

    pub fn double_type(&self) -> LLVMTypeRef {
        unsafe { LLVMDoubleTypeInContext(self.context) }
    }

    pub fn const_double(&self, val: f64) -> LLVMValueRef {
        unsafe {
            let double_type = LLVMDoubleTypeInContext(self.context);
            LLVMConstReal(double_type, val)
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

    pub fn create_binary(
        &self,
        left: LLVMValueRef,
        right: LLVMValueRef,
        name: &str,
        op: OpSymbol,
    ) -> LLVMValueRef {
        unsafe {
            let name = CString::new(name).unwrap();

            match op {
                OpSymbol::Add => LLVMBuildFAdd(self.builder, left, right, name.as_ptr()),
                OpSymbol::Sub => LLVMBuildFSub(self.builder, left, right, name.as_ptr()),
                OpSymbol::Mul => LLVMBuildFMul(self.builder, left, right, name.as_ptr()),
                OpSymbol::Div => LLVMBuildFDiv(self.builder, left, right, name.as_ptr()),
                OpSymbol::Less | OpSymbol::Greater => {
                    let cmp = if let OpSymbol::Less = op {
                        LLVMBuildFCmp(
                            self.builder,
                            LLVMRealPredicate::LLVMRealULT,
                            left,
                            right,
                            name.as_ptr(),
                        )
                    } else {
                        LLVMBuildFCmp(
                            self.builder,
                            LLVMRealPredicate::LLVMRealULT,
                            right,
                            left,
                            name.as_ptr(),
                        )
                    };
                    let bool_name = CString::new("booltmp").unwrap();
                    LLVMBuildUIToFP(
                        self.builder,
                        cmp,
                        LLVMDoubleTypeInContext(self.context),
                        bool_name.as_ptr(),
                    )
                }
            }
        }
    }

    pub fn create_call(
        &mut self,
        call: &str,
        args: &Vec<ExprAst>,
        name: &str,
    ) -> Result<LLVMValueRef> {
        unsafe {
            let call = CString::new(call).unwrap();
            let function = LLVMGetNamedFunction(self.module, call.as_ptr());
            if function.is_null() {
                return Err(
                    CompileError::UnknowFunction(call.to_string_lossy().to_string()).into(),
                );
            }

            let args_size = LLVMCountParams(function);
            if args_size as usize != args.len() {
                return Err(CompileError::IncorrectArguments {
                    expect: args_size as usize,
                    get: args.len(),
                }
                .into());
            }

            let mut args_val = Vec::new();
            let rc = RefCell::new(self);
            for arg in args {
                let arg_v = arg.codegen(&mut rc.borrow_mut())?;
                if arg_v.is_null() {
                    return Err(CompileError::FunctionArgumentIsNull.into());
                }
                args_val.push(arg_v);
            }

            let name = CString::new(name).unwrap();
            let builder = rc.borrow().builder;
            Ok(LLVMBuildCall2(
                builder,
                LLVMGlobalGetValueType(function),
                function,
                args_val.as_mut_ptr(),
                args_size,
                name.as_ptr(),
            ))
        }
    }

    pub fn create_proto(&mut self, name: &str, args: &Vec<String>) -> LLVMValueRef {
        unsafe {
            let mut doubles = vec![self.double_type(); args.len()];
            let function_type = LLVMFunctionType(
                self.double_type(),
                doubles.as_mut_ptr(),
                args.len() as u32,
                0,
            );

            let name = CString::new(name).unwrap();
            let function = LLVMAddFunction(self.module, name.as_ptr(), function_type);

            let param_count = LLVMCountParams(function);
            let mut func_params = Vec::with_capacity(param_count as usize);
            let ptr = func_params.as_mut_ptr();
            forget(func_params);
            LLVMGetParams(function, ptr);
            let func_params = Vec::from_raw_parts(ptr, param_count as usize, param_count as usize);

            for (idx, arg) in func_params.into_iter().enumerate() {
                let name = CString::new(args[idx].as_str()).unwrap();
                LLVMSetValueName2(arg, name.as_ptr(), args[idx].len());
            }

            function
        }
    }

    pub fn create_function(&mut self, func_ast: &FunctionAst) -> Result<LLVMValueRef> {
        unsafe {
            let name = CString::new(func_ast.proto.name.as_str()).unwrap();
            let mut function = LLVMGetNamedFunction(self.module, name.as_ptr());

            if function.is_null() {
                function = func_ast.proto.codegen(self)?;
            }

            if function.is_null() {
                return Err(CompileError::PointerIsNull.into());
            }

            if LLVMCountBasicBlocks(function) >= 1 {
                return Err(CompileError::FunctionRedifined.into());
            }

            let name = CString::new("entry").unwrap();
            let basic_block = LLVMAppendBasicBlockInContext(self.context, function, name.as_ptr());

            LLVMPositionBuilderAtEnd(self.builder, basic_block);

            self.names.clear();

            let param_count = LLVMCountParams(function);
            let mut func_params = Vec::with_capacity(param_count as usize);
            let ptr = func_params.as_mut_ptr();
            forget(func_params);
            LLVMGetParams(function, ptr);
            let func_params = Vec::from_raw_parts(ptr, param_count as usize, param_count as usize);

            for (idx, arg) in func_params.into_iter().enumerate() {
                self.names.insert(func_ast.proto.args[idx].clone(), arg);
            }

            match func_ast.body.codegen(self) {
                Ok(val) => {
                    LLVMBuildRet(self.builder, val);

                    LLVMVerifyFunction(
                        function,
                        llvm_sys::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
                    );

                    Ok(function)
                }
                Err(e) => {
                    LLVMEraseGlobalIFunc(function);
                    Err(e)
                }
            }
        }
    }

    pub fn print(&self, val: LLVMValueRef) -> String {
        unsafe {
            let cstring = CString::from_raw(LLVMPrintValueToString(val));
            cstring.to_string_lossy().to_string()
        }
    }

    pub fn print_module(&self) -> String {
        unsafe {
            let cstring = CString::from_raw(LLVMPrintModuleToString(self.module));
            cstring.to_string_lossy().to_string()
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_method() {
        use crate::ast::*;
        use crate::compile::Compiler;
        use crate::parser::*;

        let mut parser = Parser::new("def bar(a) foo(a, 4.0) + bar(31337);").unwrap();
        let ast = parser.parse_definition().unwrap();

        let mut compiler = Compiler::new();
        match ast.codegen(&mut compiler) {
            Ok(val) => println!("{}", compiler.print(val)),
            Err(er) => println!("{:?}", er),
        }
    }
}
