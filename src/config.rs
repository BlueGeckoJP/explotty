use std::path::Path;

use gio::glib::home_dir;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    pub ui_font_family: Option<String>,
    pub terminal_font_family: Option<String>,
    pub terminal_fallback_font_families: Option<Vec<String>>,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let toml_str = std::fs::read_to_string(path)?;
        let config: Self = toml::de::from_str(&toml_str)?;
        Ok(config)
    }

    fn generate_config_path() -> Vec<String> {
        let home_dir = home_dir();
        let config_paths = vec![
            home_dir.join(".config/explotty.toml"),
            home_dir.join(".explotty.toml"),
        ];
        config_paths
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect()
    }

    pub fn get_first_existing_path() -> Option<String> {
        let config_paths = Self::generate_config_path();
        config_paths
            .into_iter()
            .find(|path| Path::new(&path).exists())
    }
}
