use crate::package;
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::semantic_version::SemanticVersion;
use crate::storage::driver::StorageDriver;
use regex::Regex;
use std::error::Error;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub(crate) struct Index {
    packages:
        std::collections::HashMap<String, std::collections::HashMap<String, package::Package>>,
}

impl Index {
    pub(crate) async fn from_storage_bucket(
        driver: &Box<dyn StorageDriver>,
        bucket: &str,
    ) -> Result<Index, Box<dyn Error>> {
        let pattern =
            Regex::new("([0-9a-zA-Z_]+)/[0-9a-zA-Z_]+-(\\d+\\.\\d+\\.\\d+)+\\.tar\\.gz").unwrap();
        let mut index = Index::new();

        for object in driver.list(bucket).await? {
            let fields = if let Some(capture) = pattern.captures(&object) {
                (capture.get(1), capture.get(2))
            } else {
                continue;
            };

            match fields {
                (Some(name), Some(version)) => {
                    index.add(Package {
                        name: name.as_str().to_string(),
                        version: PackageVersion::Literal(version.as_str().to_string()),
                    });
                }
                _ => (),
            };
        }

        Result::Ok(index)
    }

    pub(crate) fn new() -> Index {
        Index {
            packages: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn contains(&self, package: &package::Package) -> bool {
        match self.packages.get(&package.name) {
            None => false,
            Some(items) => {
                if package.version == PackageVersion::Latest {
                    !items.is_empty()
                } else {
                    items.contains_key(&package.version.to_string())
                }
            }
        }
    }

    pub(crate) fn add(&mut self, package: Package) {
        if !self.packages.contains_key(&package.name) {
            self.packages
                .insert(package.name.clone(), std::collections::HashMap::new());
        }

        self.packages
            .get_mut(&package.name)
            .unwrap()
            .insert(package.version.literal().unwrap(), package);
    }

    pub(crate) fn dump(&self) {
        println!("Dumping package index");
        for (name, versions) in self.packages.iter() {
            for (version, package) in versions.iter() {
                println!(" - {}@{}", name, version);
            }
        }
    }

    pub(crate) fn get_latest(&self, name: &String) -> Option<SemanticVersion> {
        self.packages
            .get(name)
            .map(|x| {
                let versions: Vec<SemanticVersion> = x
                    .values()
                    .map(|x| x.to_owned())
                    .flat_map(|x| match x.version {
                        PackageVersion::Latest => None,
                        PackageVersion::Literal(v) => SemanticVersion::from_str(&v).ok(),
                    })
                    .collect();

                versions.split_first().map(|(first, rest)| {
                    rest.into_iter()
                        .fold(
                            first,
                            |result, next| {
                                if result > next {
                                    result
                                } else {
                                    next
                                }
                            },
                        )
                        .clone()
                })
            })
            .flatten()
    }
}
