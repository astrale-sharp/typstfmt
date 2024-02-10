use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(flatten)]
    pub indent: IdentSpace,
    pub max_line_length: usize,
    /// If enabled, when breaking arguments, it will try to keep more on one line.
    pub experimental_args_breaking_consecutive: bool,
    pub line_wrap: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub enum IdentSpace {
    Tabs,
    Spaces(usize),
}

impl IdentSpace {
    pub fn get(&self, level: usize) -> String {
        match self {
            IdentSpace::Tabs => "\t".repeat(level),
            IdentSpace::Spaces(n) => " ".repeat(*n).repeat(level),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // this being strictly > to 1 is assumed.
            indent: IdentSpace::Spaces(2),
            max_line_length: 80,
            line_wrap: false,
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
