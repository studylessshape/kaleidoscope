use std::fmt::Display;

use crate::lex::Token;

#[derive(Debug)]
pub enum Error {
    LexError(LexError),
    ParserError(ParserError),
    CompileError(CompileError)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

macro_rules! impl_error_from {
    ($for:ty, $($target:path => $err:ty),+) => {
        $(
            impl From<$err> for $for {
                fn from(value: $err) -> Self {
                    $target(value)
                }
            }
        )+
    };
}

macro_rules! impl_error {
    ($($err:ty),+) => {
        $(
            impl std::error::Error for $err { }
        )+
    };
}

impl_error!(Error);

impl_error_from!(
    Error,
    Error::LexError => LexError,
    Error::ParserError => ParserError,
    Error::CompileError => CompileError
);

#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("unexpected symbol: {0}")]
    UnsupportSymbol(char),
    #[error("string unclosed, expected close by: {0}")]
    UnclosedString(char),
    #[error("parse float occurs error: {0}")]
    ParseFloatError(std::num::ParseFloatError),
    #[error("occurs io error: {0}")]
    IoError(std::io::Error),
}

impl_error_from!(
    LexError,
    LexError::IoError => std::io::Error,
    LexError::ParseFloatError => std::num::ParseFloatError
);

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("{self:?}")]
    ExpectedFunctionName,
    #[error("token '{0:?}' is unsupported operator")]
    ParseOpSymbolError(Token),
    #[error("unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("{0}")]
    SyntaxError(String),
}

impl ParserError {
    pub fn syn_err<T, S>(err: S) -> crate::Result<T>
    where
        S: AsRef<[u8]>,
    {
        Err(Self::SyntaxError(String::from_utf8_lossy(err.as_ref()).to_string()).into())
    }
}

#[derive(Debug)]
pub enum CompileError {
    PointerIsNull,
    UnknowVariableName(String),
    UnknowFunction(String),
    IncorrectArguments{expect: usize, get: usize},
    FunctionArgumentIsNull,
    FunctionRedifined,
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl_error!(CompileError);