use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};
use simple_error::bail;
use std::error::Error;

pub(crate) async fn get_s3_client() -> Result<Client, Box<dyn Error>> {
    let region_name = "us-west-2";
    let region_provider = RegionProviderChain::first_try(Region::new(region_name));
    match region_provider.region().await {
        None => bail!(format!(
            "the AWS region provided is invalid: {}",
            region_name
        )),
        Some(_) => Ok(Client::new(
            &aws_config::from_env().region(region_provider).load().await,
        )),
    }
}
