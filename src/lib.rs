use std::result;

use error::Error;

pub mod lex;
pub mod error;
pub mod ast;
pub mod parser;
pub mod compile;
pub mod jit;
pub mod analysis;

type Result<T> = result::Result<T, Error>;

pub(crate) fn bool_to_i32(b: bool) -> i32 {
    if b {1} else {0}
}