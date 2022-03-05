use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) enum ParseSemanticVersionError {
    InvalidFormat,
    NonNumericSegment,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SemanticVersion(usize, usize, usize);

impl FromStr for SemanticVersion {
    type Err = ParseSemanticVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split('.').into_iter().map(|s| s.to_string()).collect();
        let mut major: usize = 0;
        let mut minor: usize = 0;
        let mut patch: usize = 0;

        for x in parts.iter().enumerate() {
            match x.0 {
                0 => {
                    major = (*x.1)
                        .parse()
                        .map_err(|_| ParseSemanticVersionError::NonNumericSegment)?;
                }
                1 => {
                    minor = (*x.1)
                        .parse()
                        .map_err(|_| ParseSemanticVersionError::NonNumericSegment)?;
                }
                2 => {
                    patch = (*x.1)
                        .parse()
                        .map_err(|_| ParseSemanticVersionError::NonNumericSegment)?;
                }
                _ => return Result::Err(ParseSemanticVersionError::InvalidFormat),
            }
        }

        Result::Ok(SemanticVersion(major, minor, patch))
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if other.0 != self.0 {
            Some(self.0.cmp(&other.0))
        } else if other.1 != self.1 {
            Some(self.1.cmp(&other.1))
        } else {
            Some(self.2.cmp(&other.2))
        }
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
