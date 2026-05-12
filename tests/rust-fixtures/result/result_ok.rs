// Test fixture: result_ok.rs - Result type testing
// @verify chimera-rust-schema
// @expected ch_status SUCCESS

use std::result::Result;

#[derive(Debug)]
pub enum MyError {
    InvalidInput(String),
    OutOfBounds { index: usize, len: usize },
    IoError(u32),
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::InvalidInput(s) => write!(f, "invalid input: {}", s),
            MyError::OutOfBounds { index, len } => {
                write!(f, "index {} out of bounds for length {}", index, len)
            }
            MyError::IoError(code) => write!(f, "I/O error: {}", code),
        }
    }
}

pub type MyResult<T> = Result<T, MyError>;

pub fn parse_index(input: &str) -> MyResult<usize> {
    let value = input.trim().parse::<usize>()
        .map_err(|_| MyError::InvalidInput(input.to_string()))?;
    Ok(value)
}
