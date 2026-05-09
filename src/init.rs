use anyhow::Result;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: Database,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub path: PathBuf,
}

pub fn create_default_config_file(config_file: &Path, data_dir: &Path) -> Result<()> {
    let db = data_dir.join("tsm.db");

    let content = format!("[database]\npath = \"{}\"", db.display());

    std::fs::write(config_file, &content)?;
    Ok(())
}

pub fn get_config(config_file: &Path) -> Result<Config> {
    let content: String = fs::read_to_string(config_file)?;

    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

pub fn init_paths() -> Result<AppPaths> {
    let proj =
        ProjectDirs::from("com", "Ars Inc", "bcl").expect("Cannot determine project directories");

    let config_dir = proj.config_dir().to_path_buf();
    let data_dir = proj.data_dir().to_path_buf();
    let cache_dir = proj.cache_dir().to_path_buf();

    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&cache_dir)?;

    let app_paths: AppPaths = AppPaths {
        config_dir,
        data_dir,
        cache_dir,
    };

    Ok(app_paths)
}
