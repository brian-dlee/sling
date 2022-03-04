use crate::index::Index;
use crate::package::Package;
use crate::package_version::PackageVersion;
use crate::storage::driver::StorageDriver;
use bytes::{Buf, Bytes};
use cloud_storage::client::ObjectClient;
use cloud_storage::object::ObjectList;
use cloud_storage::sync::Client;
use cloud_storage::{Bucket, ListRequest, Object};
use futures::{SinkExt, StreamExt};
use mime::APPLICATION_OCTET_STREAM;
use regex::Regex;
use simple_error::bail;
use std::error::Error;
use std::fs::File;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct GoogleStorageDriver {
    client: Client,
}

impl GoogleStorageDriver {
    pub(crate) fn new(client: Client) -> GoogleStorageDriver {
        GoogleStorageDriver { client }
    }
}

#[async_trait::async_trait]
impl StorageDriver for GoogleStorageDriver {
    async fn list(&self, bucket: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let req = ListRequest::default();
        let object = self.client.object();
        let response = object.list(bucket, req);
        let mut items: Vec<String> = Vec::new();

        for result in response {
            for object_list in result {
                for item in object_list.items.into_iter() {
                    items.push(item.name.clone());
                }
            }
        }

        Ok(items)
    }

    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes, Box<dyn Error>> {
        let response = self.client.object().download(bucket, key)?;

        Ok(Bytes::from(response))
    }

    async fn put(&self, bucket: &str, key: &str, content: Bytes) -> Result<(), Box<dyn Error>> {
        self.client.object().create(
            bucket,
            content.to_vec(),
            key,
            APPLICATION_OCTET_STREAM.to_string().as_str(),
        )?;

        Result::Ok(())
    }
}
