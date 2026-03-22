use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{KodeError, KodeResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keymap: KeymapConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            theme: ThemeConfig::default(),
            keymap: KeymapConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> KodeResult<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| KodeError::Config(e.to_string()))
    }

    pub fn config_dir() -> PathBuf {
        dirs_or_default()
    }
}

fn dirs_or_default() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config")
        })
        .join("kode")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub font_family: String,
    pub font_size: f32,
    pub tab_width: u32,
    pub insert_spaces: bool,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub scroll_off: u32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".into(),
            font_size: 14.0,
            tab_width: 4,
            insert_spaces: true,
            line_numbers: true,
            word_wrap: false,
            scroll_off: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub path: Option<PathBuf>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "dark".into(),
            path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapConfig {
    pub mode: String,
    pub custom_path: Option<PathBuf>,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            mode: "vim".into(),
            custom_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.editor.font_size, 14.0);
        assert_eq!(config.editor.tab_width, 4);
        assert!(config.editor.line_numbers);
    }

    #[test]
    fn parse_toml() {
        let toml_str = r#"
[editor]
font_size = 16.0
tab_width = 2
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.editor.font_size, 16.0);
        assert_eq!(config.editor.tab_width, 2);
    }
}
