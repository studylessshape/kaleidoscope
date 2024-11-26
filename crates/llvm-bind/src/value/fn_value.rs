use llvm_sys::{core::LLVMCountParams, prelude::LLVMValueRef};

use crate::ty::ValueType;

use super::{AsLLVMValueRef, Value};

pub struct FnValue {
    pub(crate) inner: Value
}

impl FnValue {
    pub unsafe fn new(v_ref: LLVMValueRef) -> Self {
        Self {
            inner: Value::new(v_ref)
        }
    }

    pub fn count_params(&self) -> u32 {
        unsafe {
            LLVMCountParams(self.as_llvm_value_ref())
        }
    }

    pub fn global_value_type(&self) -> ValueType {
        self.inner.global_value_type()
    }
}

impl AsLLVMValueRef for FnValue {
    unsafe fn as_llvm_value_ref(&self) -> LLVMValueRef {
        self.inner.as_llvm_value_ref()
    }
}