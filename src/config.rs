use crate::{yaml, RuntimeConfig};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Config {
    pub(crate) default_bucket_name: Option<String>,
    pub(crate) default_pip_args: Option<String>,
    pub(crate) default_python_interpreter: Option<String>,
}

impl Config {
    pub fn load(path: &std::path::PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
        if path.exists() {
            yaml::read_yaml(&path)
        } else {
            Result::Ok(Config {
                default_bucket_name: None,
                default_pip_args: None,
                default_python_interpreter: None,
            })
        }
    }

    pub fn as_runtime_config(&self) -> RuntimeConfig {
        RuntimeConfig {
            bucket: self.default_bucket_name.clone(),
            pip_args: self.default_pip_args.clone(),
            python: self.default_python_interpreter.clone(),
        }
    }

    pub fn save(&self, path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        yaml::write_yaml(&path, self)
    }
}
