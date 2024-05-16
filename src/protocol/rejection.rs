use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct RejectError {
    details: String,
}

impl RejectError {
    pub fn new(msg: &str) -> RejectError {
        RejectError{details: msg.to_string()}
    }
}

impl fmt::Display for RejectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for RejectError {
    fn description(&self) -> &str {
        &self.details
    }
}