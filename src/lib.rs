use std::result;

use error::Error;

pub mod lex;
pub mod error;
pub mod expr;
pub mod parser;

type Result<T> = result::Result<T, Error>;