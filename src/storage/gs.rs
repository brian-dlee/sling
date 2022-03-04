use crate::storage::driver::StorageDriver;
use crate::storage::object_ref::ObjectRef;
use bytes::Bytes;
use google_storage1::api::Object;
use google_storage1::Storage as Client;
use hyper::body::HttpBody;
use hyper::Body;
use mime::APPLICATION_OCTET_STREAM;
use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub(crate) enum GoogleStorageError {
    ErrorAndCode(String, u16),
    GeneralError(google_storage1::Error),
}

impl std::fmt::Display for GoogleStorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ErrorAndCode(msg, code) => write!(f, "{:?}: Code={} - {}", self, code, msg),
            Self::GeneralError(e) => write!(f, "{:?}: {}", self, e),
        }
    }
}

impl std::error::Error for GoogleStorageError {}

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
    async fn list(&self, bucket: &str) -> Result<Vec<ObjectRef>, Box<dyn Error>> {
        let (_, objects) = handle_error(self.client.objects().list(bucket).doit().await)?;
        let mut items: Vec<ObjectRef> = Vec::new();

        for result in objects.items {
            for object in result {
                if let Some(name) = object.name {
                    items.push(self.get_object_ref(bucket, name.as_str()));
                }
            }
        }
        Ok(items)
    }

    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes, Box<dyn Error>> {
        let object = urlencoding::encode(key).into_owned();
        let (mut response, _) = handle_error(
            self.client
                .objects()
                .get(bucket, object.as_str())
                .param("alt", "media")
                .doit()
                .await,
        )?;
        let body: &mut Body = response.body_mut();
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(data) = body.data().await {
            if let Ok(content) = data {
                buffer.append(&mut content.to_vec());
            } else {
                return Err(data.err().unwrap().into());
            }
        }

        Ok(Bytes::from(buffer))
    }

    async fn put(&self, bucket: &str, key: &str, content: Bytes) -> Result<(), Box<dyn Error>> {
        let mime = APPLICATION_OCTET_STREAM.to_string().parse().unwrap();
        let reader = std::io::Cursor::new(content);
        let req = {
            let call = self.client.objects().insert(Object::default(), bucket);
            call.name(key)
        };

        handle_error(req.upload(reader, mime).await)?;

        Result::Ok(())
    }

    fn get_protocol(&self) -> &str {
        "gs"
    }
}

fn handle_error<T>(result: google_storage1::Result<T>) -> Result<T, GoogleStorageError> {
    match result {
        Ok(x) => Ok(x),
        Err(e) => Err(match e {
            google_storage1::Error::BadRequest(response) => {
                GoogleStorageError::ErrorAndCode(response.error.message, response.error.code)
            }
            e => GoogleStorageError::GeneralError(e),
        }),
    }
}
