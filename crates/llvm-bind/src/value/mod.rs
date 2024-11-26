use llvm_sys::{core::LLVMGlobalGetValueType, prelude::LLVMValueRef};

use crate::ty::ValueType;

pub mod double;
pub mod fn_value;

pub(crate) trait AsLLVMValueRef {
    unsafe fn as_llvm_value_ref(&self) -> LLVMValueRef;
}

pub struct Value {
    pub(crate) inner: LLVMValueRef,
}

impl Value {
    pub unsafe fn new(v_ref: LLVMValueRef) -> Self {
        Self {
            inner: v_ref,
        }
    }

    pub fn global_value_type(&self) -> ValueType {
        unsafe {
            let ty = LLVMGlobalGetValueType(self.inner);
            ValueType::new(ty)
        }
    }
}

impl AsLLVMValueRef for Value {
    unsafe fn as_llvm_value_ref(&self) -> LLVMValueRef {
        self.inner
    }
}