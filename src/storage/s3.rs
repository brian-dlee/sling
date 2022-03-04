use crate::index::Index;
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::storage::driver::StorageDriver;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::{Client, Region};
use bytes::{Buf, Bytes};
use regex::Regex;
use simple_error::bail;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct S3StorageDriver {
    client: Client,
}

impl S3StorageDriver {
    pub(crate) fn new(client: Client) -> S3StorageDriver {
        S3StorageDriver { client }
    }
}

#[async_trait::async_trait]
impl StorageDriver for S3StorageDriver {
    async fn list(&self, bucket: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let response = self.client.list_objects_v2().bucket(bucket).send().await?;
        Ok(response
            .contents()
            .unwrap_or_default()
            .iter()
            .map(|o| o.key().map(|s| s.to_string()))
            .flatten()
            .collect())
    }

    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes, Box<dyn Error>> {
        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
        let data = response.body.collect().await?;
        Ok(data.into_bytes())
    }

    async fn put(&self, bucket: &str, key: &str, content: Bytes) -> Result<(), Box<dyn Error>> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(content))
            .send()
            .await?;

        Result::Ok(())
    }
}
