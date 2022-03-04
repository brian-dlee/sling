use crate::storage::object_ref::ObjectRef;
use bytes::Bytes;
use std::error::Error;

#[async_trait::async_trait]
pub(crate) trait StorageDriver {
    async fn list(&self, bucket: &str) -> Result<Vec<ObjectRef>, Box<dyn Error>>;
    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes, Box<dyn Error>>;
    async fn put(&self, bucket: &str, key: &str, content: Bytes) -> Result<(), Box<dyn Error>>;
    fn get_protocol(&self) -> &str;

    fn get_object_ref(&self, bucket: &str, key: &str) -> ObjectRef {
        ObjectRef {
            bucket: bucket.to_string(),
            key: key.to_string(),
            protocol: self.get_protocol().to_string(),
        }
    }
}
