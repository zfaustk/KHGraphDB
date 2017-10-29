use std::fmt;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error { message: message.to_string() }
    }

    /// The token that did not belong.
    pub fn near(message: &str, tok: &str) -> Error {
        if tok.is_empty() {
            Error { message: format!("{} at end", message) }
        } else {
            Error { message: format!("{} near {}", message, tok) }
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
