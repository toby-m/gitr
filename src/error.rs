use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShitError { message : Option<&'static str> }

impl ShitError {
    pub fn new(message : &'static str) -> ShitError {
        return ShitError { message: Option::Some(message) }
    }

    pub fn empty() -> ShitError {
        return ShitError { message: Option::None }
    }

    pub fn as_result<T>(message : &'static str) -> Result<T, Box<Error>> {
        Result::Err(Box::new(ShitError::new(message)))
    }
}

impl Display for ShitError {
    fn fmt(&self, f : &mut Formatter) -> FmtResult {
        match self.message {
            Some(message) => write!(f, "{}", message),
            None          => write!(f, "Failed to specify error")
        }
    }
}

impl Error for ShitError {
    fn description(&self) -> &str {
        match self.message {
            Some(s) => s,
            None => "Failed to specify error"
        }
    }
    fn cause(&self) -> Option<&Error> { return None; }
}
