use std::error::Error;

pub(crate) fn get_gs_client() -> Result<cloud_storage::sync::Client, Box<dyn Error>> {
    Ok(cloud_storage::sync::Client::new()?)
}
