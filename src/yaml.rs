use crate::config;
use std::fs::File;

pub(crate) fn read_yaml(
    path: &std::path::PathBuf,
) -> Result<config::Config, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    serde_yaml::from_reader(f).map_err(|e| e.into())
}

pub(crate) fn write_yaml(
    path: &std::path::PathBuf,
    data: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::create(path)?;
    serde_yaml::to_writer(f, &data).map_err(|e| e.into())
}
