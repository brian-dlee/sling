use crate::package::Specifier;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PackageVersion {
    Latest,
    Literal(Specifier),
}

impl std::fmt::Display for PackageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Literal(v) => write!(f, "{}", v),
        }
    }
}
