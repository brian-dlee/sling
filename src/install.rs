use crate::index::Index;
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::{aws, pip, RuntimeConfig};
use std::error::Error;

use crate::storage::driver::StorageDriver;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};
use bytes::Buf;
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
enum InstallError {
    NoBucketDefined,
    PackageNotFound(String),
    VersionResolutionFailed(String),
    AWSRegionInvalid(String),
    AWSError(String),
    DownloadError(String),
    PipError(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBucketDefined => write!(f, "no bucket was provided"),
            Self::PackageNotFound(pkg) => write!(f, "package not found: {}", pkg),
            Self::VersionResolutionFailed(pkg) => write!(f, "latest version failed: {}", pkg),
            Self::AWSRegionInvalid(region) => write!(f, "invalid AWS region: {}", region),
            Self::AWSError(msg) => write!(f, "aws error: {}", msg),
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
        if !index.contains(&package) {
            return Result::Err(InstallError::PackageNotFound(package.to_string()).into());
        }

        let version = if package.version == PackageVersion::Latest {
            let latest = index
                .get_latest(&package.name)
                .map(|x| x.to_string())
                .ok_or(InstallError::VersionResolutionFailed(
                    package.name.clone().to_string(),
                ))?;

            println!(
                "Resolved package version: {}@latest -> {}@{}",
                package.name, package.name, latest
            );

            PackageVersion::Literal(latest)
        } else {
            package.version.clone()
        };

        let path = download_package(
            driver,
            bucket.as_str(),
            package.with_version(version),
            dir.path(),
        )
        .await
        .map_err(|e| {
            InstallError::DownloadError(format!("failed to download package from s3: {}", e))
        })?;

        pip::install_package(&python, &pip_args, &path).map_err(|e| {
            InstallError::PipError(format!("failed to install package with pip: {}", e))
        })?;
    }

    Result::Ok(())
}

pub(crate) async fn download_package(
    driver: &Box<dyn StorageDriver>,
    bucket: &str,
    package: Package,
    dir: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    println!("Downloading s3://{}/{}", bucket, package.object_key());

    let data = driver.get(bucket, package.object_key().as_str()).await?;
    let target = dir.join(package.filename());

    File::create(&target)?.write(data.chunk())?;

    Result::Ok(target.clone())
}
