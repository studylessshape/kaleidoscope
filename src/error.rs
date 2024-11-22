use std::fmt::Display;

use crate::lex::Token;

#[derive(Debug)]
pub enum Error {
    LexError(LexError),
    ParserError(ParserError),
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

impl_error!(Error, LexError, ParserError);

impl_error_from!(
    Error,
    Error::LexError => LexError,
    Error::ParserError => ParserError
);

#[derive(Debug)]
pub enum LexError {
    UnsupportChar(char),
    UnclosedString,
    ParseFloatError(std::num::ParseFloatError),
    IoError(std::io::Error),
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl_error_from!(
    LexError,
    LexError::IoError => std::io::Error,
    LexError::ParseFloatError => std::num::ParseFloatError
);

#[derive(Debug)]
pub enum ParserError {
    ExpectedFunctionName,
    ParseOpSymbolError(Token),
    UnknownToken(Token),
    UnclosedGroup,
    SyntaxError(String),
    UnexpectsError(String),
}

impl ParserError {
    pub fn syn_err<T, S>(err: S) -> crate::Result<T>
    where
        S: AsRef<[u8]>,
    {
        Err(Self::SyntaxError(String::from_utf8_lossy(err.as_ref()).to_string()).into())
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
