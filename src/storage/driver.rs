use bytes::Bytes;
use std::error::Error;

#[async_trait::async_trait]
pub(crate) trait StorageDriver {
    async fn list(&self, bucket: &str) -> Result<Vec<String>, Box<dyn Error>>;
    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes, Box<dyn Error>>;
    async fn put(&self, bucket: &str, key: &str, content: Bytes) -> Result<(), Box<dyn Error>>;
}
