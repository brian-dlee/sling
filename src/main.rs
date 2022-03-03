mod active_config;
mod config;
mod index;
mod install;
mod package;
mod package_version;
mod runtime_config;
mod semantic_version;
mod yaml;

use clap::Parser;
use clap::Subcommand;
use dirs;
use serde_derive::{Deserialize, Serialize};
use serde_yaml;
use std::{
    fs::File,
    io::{BufRead, Write},
    num::ParseIntError,
    str::FromStr,
};
use tempdir;

use crate::active_config::ActiveConfig;
use crate::config::Config;
use crate::runtime_config::RuntimeConfig;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    runtime: runtime_config::RuntimeConfig,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Get {
        #[clap(short, long)]
        text_files: Vec<String>,

        packages: Vec<package::Package>,
    },

    Put {
        package_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    let home_path = match dirs::home_dir() {
        Some(path) => path,
        None => {
            return Result::Err(
                "Unable to resolve the home directory on the current system.".to_string(),
            )
        }
    };

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

    println!("{:?}", runtime_config);

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

            match install::install(&runtime_config, packages).await {
                Result::Ok(_) => (),
                Result::Err(e) => {
                    return Result::Err(format!("Failed to install package. Error={:?}", e))
                }
            }
        }
        Commands::Put { package_path } => {
            let path = std::path::PathBuf::from(package_path);

            if !path.exists() {
                return Result::Err(format!(
                    "Python package not found at provided location. Path={:?}",
                    path
                ));
            }
        }
    }

    Ok(())
}
