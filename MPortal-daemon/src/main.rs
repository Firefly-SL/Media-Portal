 #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher, event::{ModifyKind, RenameMode}};
use std::{path::Path, sync::mpsc::channel, thread, time::Duration};

mod utils;

fn main() -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(Path::new("/home/user-zero/Desktop/expirement/input"), RecursiveMode::Recursive)?;
    
    println!("watching folder...");

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let EventKind::Modify(ModifyKind::Name(RenameMode::To)) = event.kind {
                    if let Some(path) = event.paths.get(0) {
                        // let media = utils::is_media_file(&path.display().to_string());
                        if utils::is_media_file(&path.display().to_string()).0 {
                            thread::sleep(Duration::from_millis(500));
                            
                            println!("To: {:?}", path.display());
                            let output_file = utils::get_output("/home/user-zero/Desktop/expirement/output/", &path.display().to_string(), "mp4");
                            utils::media_normal_convert(&path, utils::string_to_str_slice(""), &output_file.unwrap());
                        }
                    }
                }
            },
            Ok(Err(e)) => {
                println!("notify error: {:?}", e);
            }
            Err(e) => {
                println!("channel received error: {:?}", e);
            }
        }
    }
}