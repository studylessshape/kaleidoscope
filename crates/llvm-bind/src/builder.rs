use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildCall2, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFSub,
        LLVMBuildUIToFP,
    },
    prelude::{LLVMBuilderRef, LLVMValueRef},
    LLVMRealPredicate,
};

use crate::{
    context::Context,
    value::{fn_value::FnValue, AsLLVMValueRef, Value},
};

pub struct Builder {
    pub(crate) inner: LLVMBuilderRef,
}

impl Builder {
    pub unsafe fn new(builder: LLVMBuilderRef) -> Self {
        Self { inner: builder }
    }

    pub fn build_float_add<S>(&self, lhs: &Value, rhs: &Value, name: S) -> Value
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let name = CString::new(name.as_ref()).unwrap();
            let v_ref = LLVMBuildFAdd(self.inner, lhs.inner, rhs.inner, name.as_ptr());
            Value::new(v_ref)
        }
    }

    pub fn build_float_sub<S>(&self, lhs: &Value, rhs: &Value, name: S) -> Value
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let name = CString::new(name.as_ref()).unwrap();
            let v_ref = LLVMBuildFSub(self.inner, lhs.inner, rhs.inner, name.as_ptr());
            Value::new(v_ref)
        }
    }

    pub fn build_float_mul<S>(&self, lhs: &Value, rhs: &Value, name: S) -> Value
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let name = CString::new(name.as_ref()).unwrap();
            let v_ref = LLVMBuildFMul(self.inner, lhs.inner, rhs.inner, name.as_ptr());
            Value::new(v_ref)
        }
    }

    pub fn build_float_div<S>(&self, lhs: &Value, rhs: &Value, name: S) -> Value
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let name = CString::new(name.as_ref()).unwrap();
            let v_ref = LLVMBuildFDiv(self.inner, lhs.inner, rhs.inner, name.as_ptr());
            Value::new(v_ref)
        }
    }

    pub fn build_compare_less<S>(
        &self,
        context: &Context,
        lhs: &Value,
        rhs: &Value,
        name: S,
    ) -> Value
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let name = CString::new(name.as_ref()).unwrap();
            let cmp = LLVMBuildFCmp(
                self.inner,
                LLVMRealPredicate::LLVMRealULT,
                lhs.inner,
                rhs.inner,
                name.as_ptr(),
            );
            let bool_name = CString::new("booltmp").unwrap();
            let v_ref = LLVMBuildUIToFP(
                self.inner,
                cmp,
                context.double_type().inner,
                bool_name.as_ptr(),
            );
            Value::new(v_ref)
        }
    }

    pub fn build_call<S>(&self, fn_val: &FnValue, args: &Vec<Value>, name: S) -> Value
    where
        S: AsRef<[u8]>,
    {
        unsafe {
            let arg_size = fn_val.count_params();
            let mut args: Vec<LLVMValueRef> = args.iter().map(|v| v.as_llvm_value_ref()).collect();

            let name = CString::new(name.as_ref()).unwrap();
            let v_ref = LLVMBuildCall2(
                self.inner,
                fn_val.global_value_type().inner,
                fn_val.as_llvm_value_ref(),
                args.as_mut_ptr(),
                arg_size,
                name.as_ptr(),
            );
            Value::new(v_ref)
        }
    }
}
