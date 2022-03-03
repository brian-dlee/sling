use crate::package_version::PackageVersion::Literal;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PackageVersion {
    Latest,
    Literal(String),
}

impl PackageVersion {
    pub(crate) fn literal(&self) -> Option<String> {
        match self {
            Literal(value) => Some(value.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for PackageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Literal(v) => write!(f, "{}", v),
        }
    }
}
