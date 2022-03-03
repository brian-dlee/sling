use crate::package::Package;
use crate::{s3, RuntimeConfig};
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug)]
enum PublishError {
    NoBucketDefined,
    AWSRegionInvalid(String),
    AWSError(String),
    InvalidPackage(String),
    OverwriteDisallowedError,
    UploadError(String),
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NoBucketDefined => write!(f, "no bucket was provided"),
            Self::AWSRegionInvalid(region) => write!(f, "invalid AWS region: {}", region),
            Self::AWSError(msg) => write!(f, "aws error: {}", msg),
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
    path: &PathBuf,
    overwrite: bool,
) -> Result<(), Box<dyn Error>> {
    let bucket = if config.bucket.is_none() {
        return Result::Err(PublishError::NoBucketDefined.into());
    } else {
        config.bucket.clone().unwrap()
    };

    let client = s3::get_s3_client().await?;
    let index = s3::build_package_index(&client, bucket.as_str()).await?;
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

    upload_package(&client, bucket.as_str(), path, &package).await?;

    Result::Ok(())
}

async fn upload_package(
    client: &Client,
    bucket: &str,
    path: &PathBuf,
    package: &Package,
) -> Result<(), Box<dyn Error>> {
    println!(
        "Uploading package to S3: {} -> s3://{}/{}",
        path.to_str().unwrap().to_string(),
        bucket,
        package.object_key()
    );

    let body = ByteStream::from_path(path).await.map_err(|e| {
        PublishError::UploadError(format!("failed to read package file: {}", e.to_string()))
    })?;

    client
        .put_object()
        .bucket(bucket)
        .key(package.object_key())
        .body(body)
        .send()
        .await?;

    Result::Ok(())
}
