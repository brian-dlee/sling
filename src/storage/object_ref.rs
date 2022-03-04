#[derive(Clone, Debug)]
pub(crate) struct ObjectRef {
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) protocol: String,
}

impl ObjectRef {
    pub(crate) fn get_object_url(&self) -> String {
        format!("{}://{}/{}", self.protocol, self.bucket, self.key)
    }
}
