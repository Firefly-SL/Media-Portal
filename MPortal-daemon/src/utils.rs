use std::path::{Path, PathBuf};
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
            let mut out_path = PathBuf::from(main_path);
            let file_name = format!("{}.converted.{}", stem_str, ext);
            out_path.push(file_name);
            return Some(out_path.to_string_lossy().to_string());
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
    let conversion = Command::new("ffmpeg")
            .arg("-i")
            .arg(input)
            .args(args)
            .arg(&temp_output)
            .args(&["-loglevel", "warning", "-y"])
            .status();
    
    match conversion {
        Ok(status) => {
             if status.success() {
                println!("conversion done: {}", output);
            } else {
                eprintln!("ffmpeg failed for {}", input.display());
            }
        },
        Err(e) => eprintln!("Failed to execute ffmpeg: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_output_handling() {
        let main_path = "output_folder";
        let path = "/tmp/test_video.mp4";
        let ext = "mkv";
        
        let result = get_output(main_path, path, ext).unwrap();
        
        let expected = PathBuf::from("output_folder").join("test_video.converted.mkv");
        assert_eq!(result, expected.to_string_lossy()); 
    }
}
