use llvm_sys::prelude::LLVMTypeRef;

pub mod double;

pub struct ValueType {
    pub(crate) inner: LLVMTypeRef
}

impl ValueType {
    pub unsafe  fn new(ty_ref: LLVMTypeRef) -> Self {
        Self { inner: ty_ref }
    }
}