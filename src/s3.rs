use crate::index::Index;
use crate::package::Package;
use crate::package_version::PackageVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};
use bytes::Buf;
use regex::Regex;
use simple_error::bail;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) async fn get_s3_client() -> Result<Client, std::boxed::Box<dyn std::error::Error>> {
    let region_name = "us-west-2";
    let region_provider = RegionProviderChain::first_try(Region::new(region_name));
    match region_provider.region().await {
        None => Result::Err(bail!(format!(
            "the AWS region provided is invalid: {}",
            region_name
        ))),
        Some(r) => Result::Ok(Client::new(
            &aws_config::from_env().region(region_provider).load().await,
        )),
    }
}

pub(crate) async fn build_package_index(
    client: &Client,
    bucket: &str,
) -> Result<Index, std::boxed::Box<dyn std::error::Error>> {
    let mut index = Index::new();

    for package in list_available_packages(&client, bucket).await? {
        index.add(package);
    }

    Result::Ok(index)
}

pub(crate) async fn download_package(
    client: &Client,
    bucket: &str,
    package: Package,
    dir: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    println!("Downloading s3://{}/{}", bucket, package.object_key());

    let response = client
        .get_object()
        .bucket(bucket)
        .key(package.object_key().as_str())
        .send()
        .await?;

    let data = response.body.collect().await?;

    let target = dir.join(package.filename());

    File::create(&target)?.write(data.into_bytes().chunk())?;

    Result::Ok(target.clone())
}

async fn list_available_packages(
    client: &Client,
    bucket: &str,
) -> Result<Vec<Package>, Box<dyn Error>> {
    let pattern =
        Regex::new("([0-9a-zA-Z_]+)/[0-9a-zA-Z_]+-(\\d+\\.\\d+\\.\\d+)+\\.tar\\.gz").unwrap();

    let response = client.list_objects_v2().bucket(bucket).send().await?;
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
