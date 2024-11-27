use std::{ffi::CString, result};

use error::Error;
use llvm_sys::{error::{LLVMErrorRef, LLVMErrorSuccess, LLVMGetErrorMessage}, prelude::LLVMBool};

pub mod lex;
pub mod error;
pub mod ast;
pub mod parser;
pub mod compile;
pub mod jit;
pub mod analysis;
pub mod target;

type Result<T> = result::Result<T, Error>;

/// see [Marshaling LLVMBool#llvmbool](https://ubiquitydotnet.github.io/Llvm.NET/v8.0.1/articles/InternalDetails/marshal-LLVMBool.html#llvmbool)
/// 
/// This page say:
/// 
/// > This is the traditional boolean value where 0==false and non-zero is true and uses the standard boolean marshaling support for System.Boolean
/// 
/// But in fact, the zero is true in llvm-sys.
/// 
/// Update:
/// 
/// I found some clues in [target.c#L33](https://gitlab.com/taricorp/llvm-sys.rs/-/blob/main/wrappers/target.c?ref_type=heads#L33).
/// 
/// You can see the comment:
/// 
/// ```c
/// /* These functions return true on failure. */
/// LLVMBool LLVM_InitializeNativeTarget(void) {
/// ```
/// 
/// So `true` is failure, and `zero` is success
const LLVM_SUCCESS: LLVMBool = LLVMErrorSuccess;

pub(crate) fn bool_to_llvm(b: bool) -> LLVMBool {
    if b {LLVM_SUCCESS} else {1}
}

pub(crate) unsafe fn get_error_msg(err: LLVMErrorRef) -> String {
    CString::from_raw(LLVMGetErrorMessage(err)).to_string_lossy().into_owned()
}