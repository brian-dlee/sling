use crate::storage::driver::StorageDriver;
use crate::storage::object_ref::ObjectRef;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;
use std::error::Error;

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
    async fn list(&self, bucket: &str) -> Result<Vec<ObjectRef>, Box<dyn Error>> {
        let response = self.client.list_objects_v2().bucket(bucket).send().await?;
        Ok(response
            .contents()
            .unwrap_or_default()
            .iter()
            .map(|o| o.key().map(|s| self.get_object_ref(bucket, s)))
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

    fn get_protocol(&self) -> &str {
        "s3"
    }
}
