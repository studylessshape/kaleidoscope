use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMConstReal, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext,
        LLVMModuleCreateWithNameInContext,
    },
    prelude::LLVMContextRef,
};

use crate::{builder::Builder, module::Module, ty::double::DoubleType, value::double::DoubleValue};

pub struct Context {
    pub(crate) inner: LLVMContextRef,
}

impl Context {
    pub fn double_type(&self) -> DoubleType {
        unsafe {
            let d_type = LLVMDoubleTypeInContext(self.inner);
            DoubleType::new(d_type)
        }
    }

    pub fn const_double(&self, d: f64) -> DoubleValue {
        let d_type = self.double_type();
        unsafe {
            let d_val = LLVMConstReal(d_type.inner, d);
            DoubleValue::new(d_val)
        }
    }

    pub fn create_builder(&self) -> Builder {
        unsafe {
            let builder = LLVMCreateBuilderInContext(self.inner);
            Builder::new(builder)
        }
    }

    pub fn create_module<S>(&self, name: S) -> Module
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let name = CString::new(name.as_ref()).unwrap();
            let module = LLVMModuleCreateWithNameInContext(name.as_ptr(), self.inner);
            Module::new(module)
        }
    }
}
