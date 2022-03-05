mod active_config;
mod aws;
mod config;
mod gcp;
mod index;
mod install;
mod package;
mod package_version;
mod pip;
mod publish;
mod runtime_config;
mod semantic_version;
mod storage;
mod yaml;

use clap::Parser;
use clap::Subcommand;
use std::borrow::Borrow;
use std::fmt::Formatter;
use std::str::FromStr;

use crate::active_config::ActiveConfig;
use crate::config::Config;
use crate::runtime_config::RuntimeConfig;
use crate::storage::driver::StorageDriver;
use crate::storage::gs::GoogleStorageDriver;
use crate::storage::s3::S3StorageDriver;

#[derive(Clone, Debug)]
enum StorageDriverParseError {
    InvalidStorageDriver,
}

impl std::error::Error for StorageDriverParseError {}

impl std::fmt::Display for StorageDriverParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
enum AvailableStorageDrivers {
    GS,
    S3,
}

impl FromStr for AvailableStorageDrivers {
    type Err = StorageDriverParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GS" => Ok(AvailableStorageDrivers::GS),
            "S3" => Ok(AvailableStorageDrivers::S3),
            _ => Err(StorageDriverParseError::InvalidStorageDriver),
        }
    }
}

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    runtime: runtime_config::RuntimeConfig,

    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long)]
    driver: AvailableStorageDrivers,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Get {
        #[clap(short, long)]
        text_files: Vec<String>,

        packages: Vec<package::Package>,
    },

    Put {
        #[clap(short = 'y', long)]
        overwrite: bool,

        package_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    let home_path = dirs::home_dir()
        .ok_or_else(|| "Unable to resolve the home directory on the current system.".to_string())?;

    let config_path = home_path.join(".sling.yml");

    let mut config = ActiveConfig::from(Config::load(&config_path).map_err(|e| {
        format!(
            "Failed to read config file. Path={:?}, Error={:?}",
            config_path, e
        )
    })?);

    if config.get().default_python_interpreter.is_none() && args.runtime.python.is_some() {
        config.mutate(|c| c.default_python_interpreter = args.runtime.clone().python);
    }

    if config.get().default_pip_args.is_none() && args.runtime.pip_args.is_some() {
        config.mutate(|c| c.default_pip_args = args.runtime.clone().pip_args);
    }

    if config.get().default_bucket_name.is_none() && args.runtime.bucket.is_some() {
        config.mutate(|c| c.default_bucket_name = args.runtime.clone().bucket);
    }

    config.save(&config_path).map_err(|e| {
        format!(
            "Failed to write config file. Path={:?}, Error={:?}",
            config_path, e
        )
    })?;

    let runtime_config = RuntimeConfig::resolve(
        args.runtime.clone(),
        config.get().as_runtime_config(),
        RuntimeConfig::default(),
    );

    let driver: Box<dyn StorageDriver> = match args.driver {
        AvailableStorageDrivers::GS => Box::new(GoogleStorageDriver::new(
            gcp::get_gs_client()
                .await
                .map_err(|e| format!("failed to initialize gs driver: {}", e))?,
        )),
        AvailableStorageDrivers::S3 => {
            Box::new(S3StorageDriver::new(aws::get_s3_client().await.map_err(
                |e| format!("failed to initialize aws driver: {}", e),
            )?))
        }
    };

    match args.command {
        Commands::Get {
            text_files,
            mut packages,
        } => {
            for f in text_files.iter().map(|x| std::path::PathBuf::from(x)) {
                match package::read_packages_from_file(&f) {
                    Result::Ok(mut new) => packages.append(&mut new),
                    Result::Err(e) => {
                        return Result::Err(format!(
                            "Failed to read package file. Path={:?}, Error={:?}",
                            f, e
                        ))
                    }
                }
            }

            match install::install(&runtime_config, driver.borrow(), packages).await {
                Result::Ok(_) => (),
                Result::Err(e) => {
                    return Result::Err(format!("Failed to install package. Error={:?}", e))
                }
            }
        }
        Commands::Put {
            overwrite,
            package_path,
        } => {
            let path = std::path::PathBuf::from(package_path);

            if !path.exists() {
                return Result::Err(format!(
                    "Python package not found at provided location. Path={:?}",
                    path
                ));
            }

            match publish::publish(&runtime_config, driver.borrow(), &path, overwrite).await {
                Result::Ok(_) => (),
                Result::Err(e) => {
                    return Result::Err(format!("Failed to publish package. Error={}", e))
                }
            }
        }
    }

    Ok(())
}
