use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub portal: Vec<Portal>,
}

#[derive(Serialize, Deserialize)]
pub struct Portal {
    pub portal_name: String,
    pub input_folder: String,
    pub output_folder: String,
    pub output_format: String,
    pub input_options: String,
    pub output_options: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            portal: vec![Portal {
                portal_name: "".to_string(),
                input_folder: "/path/to/the/input/folder".to_string(),
                output_folder: "/path/to/the/output/folder".to_string(),
                output_format: "mp4".to_string(),
                input_options: "".to_string(),
                output_options: "".to_string(),
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