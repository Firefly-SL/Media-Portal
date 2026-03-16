#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
 
use notify::{event::{ModifyKind, RenameMode}, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{collections::HashMap, path::{Path, PathBuf}, sync::mpsc::channel, time::{Duration, Instant}};

mod utils;
mod config;

fn main() -> Result<()> {
    log!("starting MPortal-daemon...");

    let mut _config = match config::load_or_create_config() {
        Ok(c) => c,
        Err(e) => {
            err!("failed to load config: {}", e);
            return Ok(());
        }
    };
    
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    
    let config_file_path = config::config_path().unwrap_or_default();
    let mut last_config_reload = Instant::now() - Duration::from_secs(5);
    
    // track recently processed files
    let mut processed_files: HashMap<PathBuf, Instant> = HashMap::new();

    // watch config file directory
    if let Some(parent) = config_file_path.parent() {
         if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
             err!("failed to watch config directory: {:?}", e);
         } else {
             log!("watching config directory: {:?}", parent);
         }
    }

    // watch input folders from config
    fn watch_config_folders(watcher: &mut RecommendedWatcher, config: &config::Config) {
        for path_config in &config.path {
            let input_path = Path::new(&path_config.input_folder);
            if input_path.exists() && input_path.is_dir() {
                log!("watching input folder: {}", path_config.input_folder);
                if let Err(e) = watcher.watch(input_path, RecursiveMode::Recursive) {
                    err!("failed to watch input folder {}: {:?}", path_config.input_folder, e);
                }
            } else {
                err!("input folder does not exist: {}", path_config.input_folder);
            }
        }
    }

    // initial watch set up
    watch_config_folders(&mut watcher, &_config);
    log!("service running");

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                match event.kind {
                    EventKind::Create(_) | 
                    EventKind::Modify(ModifyKind::Data(_)) |
                    EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                        
                        // check for config change with debounce
                        if event.paths.iter().any(|p| p == &config_file_path) {
                            if last_config_reload.elapsed() > Duration::from_millis(1000) {
                                log!("config file changes, reloading");
                                if let Ok(new_config) = config::load_or_create_config() {
                                    _config = new_config;
                                    watch_config_folders(&mut watcher, &_config);
                                    log!("config reloaded successfully.");
                                    last_config_reload = Instant::now();
                                }
                            }
                            continue;
                        }
                        
                        // clean up old entries from processed_files
                        processed_files.retain(|_, last_time| last_time.elapsed() < Duration::from_secs(10));

                        // handle media files
                        for path in &event.paths {
                            // ignore if processed in the last 10 seconds
                            if let Some(last_time) = processed_files.get(path) {
                                if last_time.elapsed() < Duration::from_secs(10) {
                                    continue;
                                }
                            }
                            
                            // mark as currently processing/processed
                            processed_files.insert(path.to_path_buf(), Instant::now());
                            utils::handle_media_file(path, &_config);
                        }
                    }
                     _ => {}
                }
            },
            Ok(Err(e)) => {
                err!("notify error: {:?}", e);
            }
            Err(e) => {
                err!("channel received error: {:?}", e);
            }
        }
    }
}
