
use std::fmt;

pub type ParseResult = Result<(), ParseError>;

#[derive(Debug)]
pub enum ErrorCode {
    IOError = 1,
    NotImplemented = 2,
    InvalidListTerm = 3,
    NotErlangBinary = 4
//    NotUtf8Atom = 5,
}

pub struct ParseError {
    pub error_code: ErrorCode,
    pub io_error: Option<std::io::Error>,
    pub utf8_error: Option<std::str::Utf8Error>
}

impl ParseError {
    pub fn not_erlang_binary() -> ParseError {
        ParseError::common_error(ErrorCode::NotErlangBinary)
    }
    pub fn not_implemented() -> ParseError {
        ParseError::common_error(ErrorCode::NotImplemented)
    }
    pub fn invalid_list_item() -> ParseError {
        ParseError::common_error(ErrorCode::InvalidListTerm)
    }
    fn common_error(code: ErrorCode) -> ParseError {
        ParseError{
            error_code: code,
            io_error: None,
            utf8_error: None
        }
    }
}


impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}", self.error_code)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError{
            error_code: ErrorCode::IOError,
            io_error: Some(err),
            utf8_error: None
        }
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(err: std::str::Utf8Error) -> Self {
        ParseError{
            error_code: ErrorCode::IOError,
            io_error: None,
            utf8_error: Some(err)
        }
    }
}