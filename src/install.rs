use crate::index::{Entry, Index};
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::{pip, RuntimeConfig};
use std::error::Error;

use crate::storage::driver::StorageDriver;
use bytes::Buf;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
enum InstallError {
    NoBucketDefined,
    PackageNotFound(String),
    VersionResolutionFailed(String),
    DownloadError(String),
    PipError(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBucketDefined => write!(f, "no bucket was provided"),
            Self::PackageNotFound(pkg) => write!(f, "package not found: {}", pkg),
            Self::VersionResolutionFailed(pkg) => write!(f, "latest version failed: {}", pkg),
            Self::DownloadError(msg) => write!(f, "download error: {}", msg),
            Self::PipError(msg) => write!(f, "pip error: {}", msg),
        }
    }
}

impl std::error::Error for InstallError {}

pub(crate) async fn install(
    config: &RuntimeConfig,
    driver: &Box<dyn StorageDriver>,
    packages: Vec<Package>,
) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let bucket = if config.bucket.is_none() {
        return Result::Err(InstallError::NoBucketDefined.into());
    } else {
        config.bucket.clone().unwrap()
    };

    let python = config.python.clone().unwrap_or(String::from("python"));
    let pip_args = config.pip_args.clone().unwrap_or(String::from(""));

    let dir = tempdir::TempDir::new("sling-")?;
    let index = Index::from_storage_bucket(driver, bucket.as_str()).await?;

    for package in packages {
        let object = if let PackageVersion::Literal(version) = package.version {
            if let Some(result) = index.find(&package.name, &version) {
                Ok(result)
            } else {
                Err(InstallError::PackageNotFound(
                    package.name.clone().to_string(),
                ))
            }
        } else {
            if let Some(latest) = index.find_latest(&package.name) {
                println!(
                    "Resolved package version: {}@latest -> {}@{}",
                    package.name, latest.name, latest.version
                );
                Ok(latest)
            } else {
                Err(InstallError::VersionResolutionFailed(
                    package.name.clone().to_string(),
                ))
            }
        }?;

        let package = object.as_package();
        let target = dir.path().join(package.filename());

        download_package(driver, object, &target)
            .await
            .map_err(|e| {
                InstallError::DownloadError(format!("failed to download package from s3: {}", e))
            })?;

        pip::install_package(&python, &pip_args, &target).map_err(|e| {
            InstallError::PipError(format!("failed to install package with pip: {}", e))
        })?;
    }

    Result::Ok(())
}

pub(crate) async fn download_package(
    driver: &Box<dyn StorageDriver>,
    entry: Entry,
    target: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("Downloading {}", entry.object.get_object_url());

    let data = driver
        .get(entry.object.bucket.as_str(), entry.object.key.as_str())
        .await?;

    File::create(&target)?.write(data.chunk())?;

    Result::Ok(())
}
