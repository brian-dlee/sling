use crate::index::Index;
use crate::package::Package;
use crate::package_version::PackageVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region};
use bytes::Buf;
use regex::Regex;
use simple_error::bail;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) async fn get_s3_client() -> Result<Client, Box<dyn Error>> {
    let region_name = "us-west-2";
    let region_provider = RegionProviderChain::first_try(Region::new(region_name));
    match region_provider.region().await {
        None => Result::Err(bail!(format!(
            "the AWS region provided is invalid: {}",
            region_name
        ))),
        Some(r) => Result::Ok(Client::new(
            &aws_config::from_env().region(region_provider).load().await,
        )),
    }
}
