use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Use tab characters for indent
    pub hard_tabs: bool,
    /// Indent width used for computing line length; when
    /// [`hard_tabs`](Self#structfield.hard_tabs) is set to [`false`], this is
    /// the amount of spaces used for indentation
    pub indent_space: usize,
    pub max_line_length: usize,
    /// If enabled, when breaking arguments, it will try to keep more on one line.
    pub experimental_args_breaking_consecutive: bool,
    pub line_wrap: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hard_tabs: false,
            // this being strictly > to 1 is assumed.
            indent_space: 2,
            max_line_length: 80,
            line_wrap: true,
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
