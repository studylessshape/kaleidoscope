use std::ffi::CString;

use llvm_sys::{core::LLVMModuleCreateWithName, prelude::LLVMModuleRef};

pub struct Module {
    pub(crate) inner: LLVMModuleRef,
}

impl Module {
    pub unsafe fn new(module: LLVMModuleRef) -> Self {
        Self {inner: module}
    }

    pub fn new_without_context<S>(name: S) -> Self
    where 
        S: AsRef<[u8]>
    {
        unsafe {
            let raw_string = CString::new(name.as_ref()).unwrap();
            let module = LLVMModuleCreateWithName(raw_string.as_ptr());
            Self {
                inner: module
            }
        }
    }
}