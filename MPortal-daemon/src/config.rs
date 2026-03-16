use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub path: Vec<Path>,
}

#[derive(Serialize, Deserialize)]
pub struct Path {
    pub input_folder: String,
    pub output_folder: String,
    pub output_format: String,
    pub arguments: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: vec![Path {
                input_folder: "path to the input/folder".to_string(),
                output_folder: "path to the output/folder".to_string(),
                output_format: "mp4".to_string(),
                arguments: ' '.to_string(),
            }]
        }
    }
}


pub fn load_or_create_config() -> Result<Config, io::Error> {
    let path = config_path()?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string(&Config::default())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "toml"))?;

        fs::write(&path, toml)?;
    }

    let contents = fs::read_to_string(&path)?;
    toml::from_str(&contents)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "toml"))
}

pub fn config_path() -> Result<PathBuf, io::Error> {
    let mut path = base_dir()?;
    path.push("MPortal");
    path.push("config.toml");
    Ok(path)
}

#[cfg(windows)]
fn base_dir() -> Result<PathBuf, io::Error> {
    dirs::document_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "documents dir"))
}

#[cfg(not(windows))]
fn base_dir() -> Result<PathBuf, io::Error> {
    dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "config dir"))
}