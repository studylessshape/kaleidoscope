use llvm_sys::{core::LLVMDoubleType, prelude::LLVMTypeRef};

pub struct DoubleType {
    pub(crate) inner: LLVMTypeRef,
}

impl DoubleType {
    pub unsafe fn new(type_ref: LLVMTypeRef) -> Self {
        Self { inner: type_ref }
    }

    pub fn new_without_context() -> Self {
        Self {
            inner: unsafe { LLVMDoubleType() },
        }
    }
}
