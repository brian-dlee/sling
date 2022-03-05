use crate::{yaml, RuntimeConfig};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Config {
    pub(crate) default_bucket_name: Option<String>,
    pub(crate) default_pip_args: Option<String>,
    pub(crate) default_python_interpreter: Option<String>,
}

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Config, Box<dyn std::error::Error>> {
        if path.exists() {
            yaml::read_yaml(path)
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

    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        yaml::write_yaml(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_runtime_config() {
        let config = Config {
            default_bucket_name: Some("abc".to_string()),
            default_pip_args: None,
            default_python_interpreter: None,
        };
        assert_eq!(
            config.as_runtime_config(),
            RuntimeConfig {
                bucket: Some("abc".to_string()),
                pip_args: None,
                python: None,
            }
        );
    }
}
