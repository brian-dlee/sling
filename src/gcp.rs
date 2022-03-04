use std::error::Error;

pub(crate) async fn get_gs_client() -> Result<google_storage1::Storage, Box<dyn Error>> {
    let var = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").map_err(|e| {
        simple_error::SimpleError::new(format!(
            "failed to read GOOGLE_APPLICATION_CREDENTIALS: {}",
            e
        ))
    })?;
    let key = yup_oauth2::read_service_account_key(std::path::Path::new(var.as_str())).await?;
    let builder = yup_oauth2::ServiceAccountAuthenticator::builder(key);
    Ok(google_storage1::Storage::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        builder.build().await?,
    ))
}
