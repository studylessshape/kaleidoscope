use std::result;

use error::Error;

pub mod lex;
pub mod error;
pub mod ast;
pub mod parser;
pub mod code_ir;
pub mod runtime;

type Result<T> = result::Result<T, Error>;