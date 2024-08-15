use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Config {
    pub(crate) enable_wasi: bool,
    pub(crate) wasi_dirs: Vec<(PathBuf, String)>,
    pub(crate) wasi_args: Vec<String>,
    pub(crate) start_function: Option<String>,
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

    pub fn enable_wasi(mut self, enable_wasi: bool) -> Self {
        self.config.enable_wasi = enable_wasi;
        self
    }

    pub fn set_wasi_dirs(mut self, dirs: Vec<(PathBuf, String)>) -> Self {
        self = self.enable_wasi(true);
        self.config.wasi_dirs = dirs;
        self
    }

    pub fn set_wasi_args(mut self, args: Vec<String>) -> Self {
        self = self.enable_wasi(true);
        self.config.wasi_args = args;
        self
    }

    pub fn set_start_function(mut self, start_function: String) -> Self {
        self.config.start_function = Some(start_function);
        self
    }

    pub fn finish(self) -> Config {
        self.config
    }
}
