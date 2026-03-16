use std::fs;
use std::path::{Path, PathBuf};
use std::{process::Command, thread, time::{Duration}};

use crate::config;

pub fn is_media_file(path: &str) -> (bool, Option<String>) {
    let media_exts = [
        //video
        "mp4", "mov", "mkv", "avi", "flv", "wmv", "webm",
        "mpg", "mpeg", "m4v", "ts", "ogv", "rm", "3gp", "3g2", "dv", "vob", "mts", "m2ts",
        "f4v", "f4p", "f4a", "f4b", "gif", "mj2", "nut",
        
        //audio
        "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "ac3", "alac", "opus", "amr", "au",
        
        // image
        "jpg", "jpeg", "png", "bmp", "tiff", "tif", "webp", "exr",
    ];

    if let Some(ext) = Path::new(path).extension() {
        if let Some(ext_str) = ext.to_str() {
            return (media_exts.contains(&ext_str.to_lowercase().as_str()), Some(ext_str.to_lowercase()))
        }
    }
    (false, None)
}

pub fn get_output(main_path: &str, path: &str, ext: &str) -> Option<String> {
    let path = Path::new(path);
    if let Some(stem) = path.file_stem() {
        if let Some(stem_str) = stem.to_str() {
                let new_name = format!("{}{}.converted.{}", main_path, stem_str, ext);
                return Some(new_name);
        }
    }
    None
}

pub fn string_to_str_slice(item_string: &str) -> Vec<&str> {
    return item_string.split_whitespace().collect();
}

pub fn handle_media_file(path: &Path, config: &config::Config) {
    if !path.exists() { return; }

    // find matching config
    if let Some(path_config) = config.path.iter().find(|p| {
        let input_path = Path::new(&p.input_folder);
        if let Ok(canon_input) = input_path.canonicalize() {
             if path.starts_with(&canon_input) { return true; }
        }
        path.starts_with(input_path)
    }) {
        if is_media_file(&path.to_string_lossy()).0 {
            println!("detected media file: {:?}", path);
            
            // need replacing
            thread::sleep(Duration::from_millis(1500));
            if !path.exists() { return; }
 
            if let Some(output_file) = get_output(&path_config.output_folder, &path.to_string_lossy(), &path_config.output_format) {
                let args = string_to_str_slice(&path_config.arguments);
                media_normal_convert(path, args, &output_file);               
            } else {
                eprintln!("failed to determine output file path for {:?}", path);
            }
        }
    }
}

pub fn media_normal_convert(input: &Path, args: Vec<&str>, output: &str) {
    let input_parent = input.parent().unwrap_or_else(|| Path::new("."));
    
    // create a temp file in the system temp directory
    let file_name = Path::new(output).file_name().unwrap_or_default();
    let temp_output = std::env::temp_dir().join(file_name);

    println!("converting in temp folder: {:?}", temp_output);

    let conversion = Command::new("ffmpeg")
            .arg("-i")
            .arg(input)
            .args(args)
            .arg(&temp_output)
            .args(&["-loglevel", "warning", "-y"])
            .status()
            .expect("failed to execute process");
    
    match conversion {
        Ok(status) => {
             if status.success() {
                // where to move the finished file???
                let final_output_path = Path::new(output);
                let destination = if let Some(parent) = final_output_path.parent() {
                    if parent.exists() && parent.is_dir() {
                        final_output_path.to_path_buf()
                    } else {
                        // if the idiot didn't made the output path, use input directory
                        input_parent.join(file_name)
                    }
                } else {
                    input_parent.join(file_name)
                };

                println!("moving finished file to: {:?}", destination);

                if let Err(e) = fs::rename(&temp_output, &destination) {
                    eprintln!("failed to move file from temp to final destination: {}", e);
                    // try copying if rename fails
                    if let Err(e2) = fs::copy(&temp_output, &destination) {
                        eprintln!("failed to copy file from temp to final destination: {}", e2);
                    } else {
                        let _ = fs::remove_file(&temp_output);
                        println!("conversion done: {:?}", destination);
                    }
                } else {
                    println!("conversion done: {:?}", destination);
                }
            } else {
                eprintln!("ffmpeg failed for {}", input.display());
                let _ = fs::remove_file(&temp_output);
            }
        },
        Err(e) => {
            eprintln!("failed to execute ffmpeg: {}", e);
            let _ = fs::remove_file(&temp_output);
        },
    }
}