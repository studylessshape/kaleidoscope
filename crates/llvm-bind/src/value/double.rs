use llvm_sys::prelude::LLVMValueRef;

use super::Value;

pub struct DoubleValue {
    pub(crate) inner: Value
}

impl DoubleValue {
    pub unsafe fn new(v_ref: LLVMValueRef) -> Self {
        Self {
            inner: Value::new(v_ref),
        }
    }
}

impl From<Value> for DoubleValue {
    fn from(value: Value) -> Self {
        Self { inner: value }
    }
}