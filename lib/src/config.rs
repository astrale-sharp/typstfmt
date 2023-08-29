use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    pub indent_space: usize,
    pub max_line_length: usize,
    /// If enabled, when breaking arguments, it will try to keep more on one line.
    pub experimental_args_breaking_consecutive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // this being strictly > to 1 is assumed.
            indent_space: 2,
            max_line_length: 50,
            experimental_args_breaking_consecutive: false,
        }
    }
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self, String> {
        toml::from_str(s).map_err(|e| e.message().to_string())
    }

    pub fn default_toml() -> String {
        toml::to_string_pretty(&Self::default()).unwrap()
    }
}
