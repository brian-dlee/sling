use crate::index::Index;
use crate::package::Package;
use crate::{RuntimeConfig, StorageDriver};
use bytes::Bytes;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
enum PublishError {
    NoBucketDefined,
    InvalidPackage(String),
    OverwriteDisallowedError,
    UploadError(String),
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBucketDefined => write!(f, "no bucket was provided"),
            Self::InvalidPackage(path) => write!(f, "invalid package file: {}", path),
            Self::UploadError(msg) => write!(f, "package upload failed: {}", msg),
            Self::OverwriteDisallowedError => {
                write!(f, "{:?}: refusing to overwrite published package", self)
            }
        }
    }
}

impl std::error::Error for PublishError {}

pub(crate) async fn publish(
    config: &RuntimeConfig,
    driver: &'_ dyn StorageDriver,
    path: &Path,
    overwrite: bool,
) -> Result<(), Box<dyn Error>> {
    let bucket = if config.bucket.is_none() {
        return Result::Err(PublishError::NoBucketDefined.into());
    } else {
        config.bucket.clone().unwrap()
    };

    let index = Index::from_storage_bucket(driver, bucket.as_str()).await?;
    let package = match Package::from_file(path) {
        Some(x) => x,
        None => {
            return Result::Err(
                PublishError::InvalidPackage(path.to_str().unwrap().to_string()).into(),
            );
        }
    };

    if !overwrite && index.contains(&package) {
        return Result::Err(PublishError::OverwriteDisallowedError.into());
    }

    upload_package(driver, bucket.as_str(), path, &package).await?;

    Result::Ok(())
}

async fn upload_package(
    driver: &'_ dyn StorageDriver,
    bucket: &str,
    path: &Path,
    package: &Package,
) -> Result<(), Box<dyn Error>> {
    println!(
        "Uploading package to S3: {} -> s3://{}/{}",
        path.to_str().unwrap().to_string(),
        bucket,
        package.object_key()
    );

    let mut data = Vec::new();
    let mut file = File::open(path).map_err(|e| {
        PublishError::UploadError(format!("failed to read package file: {}", e.to_string()))
    })?;

    file.read_to_end(&mut data)?;
    driver
        .put(bucket, package.object_key().as_str(), Bytes::from(data))
        .await?;

    Result::Ok(())
}
