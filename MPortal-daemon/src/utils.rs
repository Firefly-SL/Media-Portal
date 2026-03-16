use std::path::{Path};
use std::process::Command;

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

pub fn media_normal_convert(input: &Path, args: Vec<&str>, output: &str) {
    #[cfg(not(windows))]
    let conversion = Command::new("ffmpeg")
            .arg("-i")
            .arg(input)
            .args(args)
            .arg(output)
            .args(&["-loglevel", "warning", "-y"])
            .status()
            .expect("failed to execute process");
    
    if conversion.success() {
        println!("Conversion done: {}", output);
    } else {
        eprintln!("ffmpeg failed for {}", input.display());
    }
}