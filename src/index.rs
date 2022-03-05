use crate::package;
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::semantic_version::SemanticVersion;
use crate::storage::driver::StorageDriver;
use crate::storage::object_ref::ObjectRef;
use regex::Regex;
use std::error::Error;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) object: ObjectRef,
}

impl Entry {
    pub(crate) fn new(name: &str, version: &str, object: &ObjectRef) -> Entry {
        Entry {
            name: name.to_string(),
            version: version.to_string(),
            object: object.clone(),
        }
    }

    pub(crate) fn as_package(&self) -> Package {
        Package {
            name: self.name.clone(),
            version: PackageVersion::Literal(self.version.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Index {
    entries: std::collections::HashMap<String, std::collections::HashMap<String, Entry>>,
}

impl Index {
    pub(crate) async fn from_storage_bucket(
        driver: &'_ dyn StorageDriver,
        bucket: &str,
    ) -> Result<Index, Box<dyn Error>> {
        let pattern =
            Regex::new("([0-9a-zA-Z_]+)/[0-9a-zA-Z_]+-(\\d+\\.\\d+\\.\\d+)+\\.tar\\.gz").unwrap();
        let mut index = Index::new();

        for object in driver.list(bucket).await? {
            let fields = if let Some(capture) = pattern.captures(object.key.as_str()) {
                (capture.get(1), capture.get(2))
            } else {
                continue;
            };

            if let (Some(name), Some(version)) = fields {
                index.add(Entry::new(name.as_str(), version.as_str(), &object));
            };
        }

        Result::Ok(index)
    }

    pub(crate) fn new() -> Index {
        Index {
            entries: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn contains(&self, package: &package::Package) -> bool {
        match self.entries.get(&package.name) {
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

    pub(crate) fn add(&mut self, entry: Entry) {
        if !self.entries.contains_key(&entry.name) {
            self.entries
                .insert(entry.name.clone(), std::collections::HashMap::new());
        }

        self.entries
            .get_mut(&entry.name)
            .unwrap()
            .insert(entry.clone().version, entry.clone());
    }

    pub(crate) fn find_latest(&self, name: &str) -> Option<Entry> {
        if let Some((first, remaining)) = self.get_available_versions(name).split_first() {
            let result = remaining
                .iter()
                .fold(first, |a, b| if a.0 > b.0 { a } else { b });
            Some(result.to_owned().1)
        } else {
            None
        }
    }

    pub(crate) fn find(&self, name: &str, version: &str) -> Option<Entry> {
        for x in self.get_available_versions(name) {
            if x.0.to_string() == *version {
                return Some(x.1);
            }
        }
        None
    }

    fn get_available_versions(&self, name: &str) -> Vec<(SemanticVersion, Entry)> {
        if let Some(entries) = self.entries.get(name) {
            entries
                .values()
                .flat_map(|x| {
                    SemanticVersion::from_str(&x.version)
                        .map(|v| (v, x.clone()))
                        .ok()
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
