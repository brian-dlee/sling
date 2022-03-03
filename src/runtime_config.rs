use clap::clap_derive::Parser;

#[derive(Parser, Clone, Debug, PartialEq)]
pub(crate) struct RuntimeConfig {
    #[clap(long)]
    pub(crate) pip_args: Option<String>,

    #[clap(short, long)]
    pub(crate) python: Option<String>,

    #[clap(short, long)]
    pub(crate) bucket: Option<String>,
}

impl RuntimeConfig {
    pub(crate) fn default() -> RuntimeConfig {
        RuntimeConfig {
            bucket: None,
            pip_args: None,
            python: None,
        }
    }

    pub(crate) fn resolve(
        supplied: RuntimeConfig,
        stored: RuntimeConfig,
        defaults: RuntimeConfig,
    ) -> RuntimeConfig {
        RuntimeConfig {
            bucket: supplied.bucket.or(stored.bucket).or(defaults.bucket),
            pip_args: supplied.pip_args.or(stored.pip_args).or(defaults.pip_args),
            python: supplied.python.or(stored.python).or(defaults.python),
        }
    }
}
