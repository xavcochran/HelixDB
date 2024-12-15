use core::fmt;

use super::helix_parser::Rule;

pub trait Parser2 {
    fn parse(&self, input: &str) -> Result<(), String>;
}

#[derive(Debug)]
pub enum ParserError {
    ParseError(String),
    LexError(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::ParseError(e) => write!(f, "Parse error: {}", e),
            ParserError::LexError(e) => write!(f, "Lex error: {}", e),
        }
    }
}

impl From<pest::error::Error<Rule>> for ParserError {
    fn from(e: pest::error::Error<Rule>) -> Self {
        ParserError::ParseError(e.to_string())
    }
}
