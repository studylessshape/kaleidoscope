use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    LexError(LexError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    
}

#[derive(Debug)]
pub enum LexError {
    ParseFloatError(std::num::ParseFloatError),
    IoError(std::io::Error)
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LexError {
    
}

impl From<std::io::Error> for LexError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::num::ParseFloatError> for LexError {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}