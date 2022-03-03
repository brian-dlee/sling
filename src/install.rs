use crate::index::Index;
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::RuntimeConfig;

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

    let client = get_s3_client().await?;

    let mut index = Index::new();

    for package in list_available_packages(&client, bucket.as_str()).await? {
        index.add(package);
    }

    index.dump();

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
            &client,
            bucket.as_str(),
            package.with_version(version),
            dir.path(),
        )
        .await?;

        install_package(&python, &pip_args, &path)?;
    }

    Result::Ok(())
}

async fn get_s3_client() -> Result<Client, InstallError> {
    let region_name = "us-west-2";
    let region_provider = RegionProviderChain::first_try(Region::new(region_name));
    match region_provider.region().await {
        None => Result::Err(InstallError::AWSRegionInvalid(region_name.to_string())),
        Some(r) => Result::Ok(Client::new(
            &aws_config::from_env().region(region_provider).load().await,
        )),
    }
}

async fn list_available_packages(
    client: &Client,
    bucket: &str,
) -> Result<Vec<Package>, InstallError> {
    let pattern =
        Regex::new("([0-9a-zA-Z_]+)/[0-9a-zA-Z_]+-(\\d+\\.\\d+\\.\\d+)+\\.tar\\.gz").unwrap();

    let response = client
        .list_objects_v2()
        .bucket(bucket)
        .send()
        .await
        .map_err(|e| InstallError::AWSError(e.to_string()))?;
    let captures = response
        .contents()
        .unwrap_or_default()
        .iter()
        .map(|o| o.key().and_then(|key| pattern.captures(key)))
        .flatten();

    Result::Ok(
        captures
            .map(|capture| match (capture.get(1), capture.get(2)) {
                (Some(name), Some(version)) => Some(Package {
                    name: name.as_str().to_string(),
                    version: PackageVersion::Literal(version.as_str().to_string()),
                }),
                _ => None,
            })
            .flatten()
            .collect(),
    )
}

async fn download_package(
    client: &Client,
    bucket: &str,
    package: Package,
    dir: &Path,
) -> Result<PathBuf, InstallError> {
    println!("Downloading s3://{}/{}", bucket, package.object_key());

    let response = client
        .get_object()
        .bucket(bucket)
        .key(package.object_key().as_str())
        .send()
        .await
        .map_err(|e| InstallError::AWSError(e.to_string()))?;

    let data = response
        .body
        .collect()
        .await
        .map_err(|e| InstallError::AWSError(e.to_string()))?;

    let target = dir.join(package.filename());
    let mut file = File::create(&target).map_err(|e| {
        InstallError::DownloadError(format!(
            "failed to create download target: {}",
            e.to_string()
        ))
    })?;

    file.write(data.into_bytes().chunk());

    Result::Ok(target.clone())
}

fn install_package(python: &String, pip_args: &String, path: &PathBuf) -> Result<(), InstallError> {
    let path = path.to_str().unwrap();
    let extra: Vec<&str> = pip_args
        .split_whitespace()
        .fold(Vec::new(), |mut result, x| {
            match x.trim() {
                arg if !arg.is_empty() => result.push(arg),
                _ => (),
            }
            result
        });

    println!("Installing {} (Interpreter={})", path, python);

    let mut child = Command::new(python)
        .args(
            ["-m", "pip", "install", "--upgrade"]
                .iter()
                .chain(extra.iter())
                .chain([path].iter()),
        )
        .spawn()
        .map_err(|e| InstallError::PipError(e.to_string()))?;

    child
        .wait()
        .map_err(|e| InstallError::PipError(e.to_string()))?;

    Ok(())
}
