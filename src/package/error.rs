use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub(crate) enum ParsePackageError {
    InvalidFormat,
}

impl ParsePackageError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::InvalidFormat => "InvalidFormat: Use the format PKG[@(x.y.z|latest)]".to_string(),
        }
    }
}

impl Display for ParsePackageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Error for ParsePackageError {}
