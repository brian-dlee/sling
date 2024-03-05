use crate::package::error::ParsePackageError;
use crate::package::patterns::{FILENAME_PACKAGE_RE, SPECIFIER_RE, STANDARD_PACKAGE_RE};
use crate::package_version::PackageVersion;
use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: PackageVersion,
}

impl Package {
    pub(crate) fn from_path(p: &PathBuf) -> Result<Self, ParsePackageError> {
        match FILENAME_PACKAGE_RE.captures(p.to_str().unwrap()) {
            Some(captures) => match (captures.name("name"), Specifier::from_captures(captures)) {
                (Some(name), Ok(specifier)) => Ok(Package {
                    name: name.as_str().to_string(),
                    version: PackageVersion::Literal(specifier),
                }),
                (_, _) => Err(ParsePackageError::InvalidFormat),
            },
            _ => Err(ParsePackageError::InvalidFormat),
        }
    }

    pub(crate) fn object_key(&self) -> String {
        format!("{}/{}", self.name, self.filename())
    }

    pub(crate) fn filename(&self) -> String {
        format!("{}.tar.gz", self.slug())
    }

    pub(crate) fn slug(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Specifier {
    pub(crate) op: String,
    pub(crate) epoch: u32,
    pub(crate) version: String,
    pub(crate) pre: (bool, u32),
    pub(crate) post: (bool, u32),
    pub(crate) dev: (bool, u32),
    pub(crate) local: (bool, String),
}

impl Specifier {
    pub(crate) fn from_captures(c: regex::Captures) -> Result<Self, ParsePackageError> {
        let (op, epoch, version, pre, pre_num, post, post_num, dev, dev_num, local) = (
            c.name("op"),
            c.name("epoch"),
            c.name("version"),
            c.name("pre"),
            c.name("pre_num"),
            c.name("post"),
            c.name("post_num"),
            c.name("dev"),
            c.name("dev_num"),
            c.name("local"),
        );
        Ok(Specifier {
            op: op.map(|x| x.as_str()).unwrap_or("==").to_string(),
            epoch: epoch.unwrap().as_str().parse().unwrap_or(0),
            version: version.unwrap().as_str().to_string(),
            pre: (
                pre.map(|x| x.as_str().len() > 0).unwrap_or(false),
                pre_num.unwrap().as_str().parse().unwrap_or(0),
            ),
            post: (
                post.map(|x| x.as_str().len() > 0).unwrap_or(false),
                post_num.unwrap().as_str().parse().unwrap_or(0),
            ),
            dev: (
                dev.map(|x| x.as_str().len() > 0).unwrap_or(false),
                dev_num.unwrap().as_str().parse().unwrap_or(0),
            ),
            local: (
                local.unwrap().as_str().len() > 0,
                local.unwrap().as_str().to_string(),
            ),
        })
    }
}

impl FromStr for Specifier {
    type Err = ParsePackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match SPECIFIER_RE.captures(s) {
            Some(captures) => Specifier::from_captures(captures),
            None => Err(Self::Err::InvalidFormat),
        }
    }
}

impl FromStr for Package {
    type Err = ParsePackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match STANDARD_PACKAGE_RE.captures(s) {
            Some(captures) => match (captures.name("name"), Specifier::from_captures(captures)) {
                (Some(name), Ok(specifier)) => Ok(Package {
                    name: name.as_str().to_string(),
                    version: PackageVersion::Literal(specifier),
                }),
                (_, _) => Err(Self::Err::InvalidFormat),
            },
            _ => Err(Self::Err::InvalidFormat),
        }
    }
}

pub(crate) fn read_packages_from_file(
    path: &Path,
) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut packages: Vec<Package> = Vec::new();

    for line in std::io::BufReader::new(file).lines() {
        packages.push(Package::from_str(line?.trim())?);
    }

    Ok(packages)
}
