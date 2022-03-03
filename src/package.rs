use crate::package_version;
use crate::package_version::PackageVersion;
use std::fs::File;
use std::io::BufRead;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) enum ParsePackageError {
    InvalidFormat,
}

impl ParsePackageError {
    pub(crate) fn to_string(&self) -> String {
        match self {
            Self::InvalidFormat => "InvalidFormat: Use the format PKG[@(x.y.z|latest)]".to_string(),
        }
    }
}

impl std::fmt::Display for ParsePackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::error::Error for ParsePackageError {
    fn description(&self) -> &str {
        match self {
            Self::InvalidFormat => "InvalidFormat",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: package_version::PackageVersion,
}

impl Package {
    pub(crate) fn object_key(&self) -> String {
        format!("{}/{}", self.name, self.filename())
    }

    pub(crate) fn filename(&self) -> String {
        format!("{}.tar.gz", self.slug())
    }

    pub(crate) fn slug(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    pub(crate) fn with_version(&self, version: PackageVersion) -> Package {
        Package {
            name: self.name.clone(),
            version,
        }
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

impl FromStr for Package {
    type Err = ParsePackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = s.split("@").into_iter().map(|s| s.to_string()).collect();
        let (name, version) = match parts {
            parts if parts.len() == 2 => (
                parts[0].clone(),
                package_version::PackageVersion::Literal(parts[1].clone()),
            ),
            parts if parts.len() == 1 => {
                (parts[0].clone(), package_version::PackageVersion::Latest)
            }
            _ => return Result::Err(ParsePackageError::InvalidFormat),
        };

        Result::Ok(Package { name, version })
    }
}

pub(crate) fn read_packages_from_file(
    path: &std::path::PathBuf,
) -> Result<Vec<Package>, std::boxed::Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut packages: Vec<Package> = Vec::new();

    for line in std::io::BufReader::new(file).lines() {
        packages.push(Package::from_str(line?.trim())?);
    }

    Result::Ok(packages)
}
