
use std::fmt;

pub type ParseResult = Result<(), ParseError>;

#[derive(Debug)]
pub enum ErrorCode {
    ReadError = 1,
    NotImplemented = 2,
    InvalidListTerm = 3,
    NotErlangBinary = 4,
    NotUtf8Atom = 5,
}

pub struct ParseError {
    error_code: ErrorCode
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}", self.error_code)
    }
}
impl ParseError {
    pub fn new(err_code: ErrorCode) -> ParseError {
        ParseError{error_code: err_code}
    }
}

impl From<std::io::Error> for ParseError {
    fn from(_err: std::io::Error) -> Self {
        ParseError::new(ErrorCode::ReadError)
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(_err: std::str::Utf8Error) -> Self {
        ParseError::new(ErrorCode::NotUtf8Atom)
    }
}