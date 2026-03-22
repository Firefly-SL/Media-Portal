#![cfg_attr(windows, windows_subsystem = "windows")]

use clap::{Parser, ArgAction};
use notify::{event::{ModifyKind, RenameMode}, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}, sync::{mpsc::channel, Arc, Mutex}, time::{Duration, Instant}};
use std::sync::atomic::Ordering;

use mportal_core::{config, utils, DEBUG_ENABLED, log, err};

#[derive(Parser)]
#[command(name = "mportal-daemon", disable_version_flag=true)]
struct CliFlags {
    #[arg(short = 'd', long = "debug", action = ArgAction::SetTrue)]
    debug: bool,
}

fn main() -> Result<()> {
    let cli_flags = CliFlags::parse();
    DEBUG_ENABLED.store(cli_flags.debug, Ordering::Relaxed);
    
    let config_file_path = config::config_path().unwrap_or_default();
    let log_dir = config_file_path.parent().unwrap_or_else(|| Path::new("."));
        
    // init loggerm ErrLog.log and DebugLog.log
    if let Err(e) = utils::init_logger(log_dir) {
        eprintln!("failed to start logger: {:?}", e);
    }

    log!("starting MPortal-daemon...");
    
    let config = match config::load_or_create_config() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            err!("failed to load config: {:?}", e);
            return Ok(());
        }
    };
    
    let mut current_config = config;
    
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    let mut last_config_reload = Instant::now() - Duration::from_secs(5);
    
    // track files currently being processed to prevent overlap, (did happened once)
    let processing_files = Arc::new(Mutex::new(HashSet::<PathBuf>::new()));
    // track recently processed files
    let mut processed_files: HashMap<PathBuf, Instant> = HashMap::new();

    // watch config file directory
    if let Some(parent) = config_file_path.parent() {
         if let Err(_e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
             err!("failed to watch config directory: {:?}", parent);
         } else {
             log!("watching config directory: {:?}", parent);
         }
    }

    // helper to watch folders
    fn watch_config_folders(watcher: &mut RecommendedWatcher, config: &config::Config) {
        for path_config in &config.portal {
            let input_path = Path::new(&path_config.input_folder);
            if input_path.exists() && input_path.is_dir() {
                log!("watching input folder: {}", path_config.input_folder);
                if let Err(e) = watcher.watch(input_path, RecursiveMode::Recursive) {
                    err!("failed to watch input folder {:?}: {:?}", path_config.input_folder, e);
                }
            } else if path_config.input_folder != "/path/to/the/input/folder" {
                    err!("input folder does not exist: {:?}", path_config.input_folder);
            }
        }
    }

    // initial helper setup
    watch_config_folders(&mut watcher, &current_config);
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
                                    current_config = Arc::new(new_config);
                                    watch_config_folders(&mut watcher, &current_config);
                                    log!("config reloaded successfully.");
                                    last_config_reload = Instant::now();
                                }
                            }
                            continue;
                        }
                        
                        // clean up old entries from processed_files
                        processed_files.retain(|_, last_time| last_time.elapsed() < Duration::from_secs(10));

                        // handle media files
                        for path in event.paths {
                            // ignore if already being processed
                            if processing_files.lock().unwrap().contains(&path) {
                                continue;
                            }

                            // ignore if processed in the last 10 seconds
                            if let Some(last_time) = processed_files.get(&path) {
                                if last_time.elapsed() < Duration::from_secs(10) {
                                    continue;
                                }
                            }
                            
                            // mark as currently processing
                            processing_files.lock().unwrap().insert(path.clone());
                            processed_files.insert(path.clone(), Instant::now());
                            
                            let path_clone = path.clone();
                            let config_clone = Arc::clone(&current_config);
                            let processing_files_clone = Arc::clone(&processing_files);
                            
                            std::thread::spawn(move || {
                                utils::handle_media_file(&path_clone, &config_clone);
                                processing_files_clone.lock().unwrap().remove(&path_clone);
                            });
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
