use crate::cluster::ClusterConfig;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Config {
    pub wasi_enabled: bool,
    pub wasi_dirs: Vec<(PathBuf, String)>,
    pub wasi_args: Vec<String>,
    pub start_function: Option<String>,
    pub cluster_config: ClusterConfig,
}

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn enable_wasi(&mut self) -> &mut Self {
        self.config.wasi_enabled = true;
        self
    }

    pub fn set_wasi_dirs(&mut self, dirs: Vec<(PathBuf, String)>) -> &mut Self {
        self.enable_wasi();
        self.config.wasi_dirs = dirs;
        self
    }

    pub fn set_wasi_args(&mut self, args: Vec<String>) -> &mut Self {
        self.enable_wasi();
        self.config.wasi_args = args;
        self
    }

    pub fn set_start_function(&mut self, start_function: String) -> &mut Self {
        self.config.start_function = Some(start_function);
        self
    }

    pub fn finish(self) -> Config {
        self.config
    }
}
