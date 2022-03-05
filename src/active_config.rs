use crate::config::Config;

#[derive(Debug)]
pub(crate) struct ActiveConfig {
    config: Config,
    dirty: bool,
}

impl ActiveConfig {
    pub fn from(config: Config) -> ActiveConfig {
        ActiveConfig {
            config,
            dirty: false,
        }
    }

    pub fn get(&self) -> &Config {
        &self.config
    }

    pub fn save(
        &self,
        path: &std::path::Path,
    ) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
        if !self.dirty {
            Ok(())
        } else {
            self.config.save(path)
        }
    }

    pub fn mutate<F: FnOnce(&mut Config)>(&mut self, f: F) -> bool {
        let previous = self.config.clone();
        f(&mut self.config);
        self.dirty = self.dirty || self.config != previous;
        self.dirty
    }
}
