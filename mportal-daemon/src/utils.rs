use notify_rust::Notification;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::{process::Command, thread, time::Duration};
use chrono::Local;

use crate::config;

static ERR_LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static DEBUG_LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

// Errlogger and --debug logger
pub fn init_logger(log_dir: &Path) -> std::io::Result<()> {
    if !log_dir.exists() {
        fs::create_dir_all(log_dir)?;
    }

    let err_log_path = log_dir.join("ErrLog.log");
    let err_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(err_log_path)?;
    let _ = ERR_LOG_FILE.set(Mutex::new(err_file));

    let debug_log_path = std::env::temp_dir().join("mportal_debug.log");
    let debug_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(debug_log_path)?;
    let _ = DEBUG_LOG_FILE.set(Mutex::new(debug_file));

    Ok(())
}

pub fn _log_to_err_file(msg: &str) {
    if let Some(mutex) = ERR_LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let _ = writeln!(file, "{}", msg);
        }
    }
}

pub fn _log_to_debug_file(msg: &str) {
    if let Some(mutex) = DEBUG_LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let _ = writeln!(file, "{}", msg);
        }
    }
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// custom made print function.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        let timestamp = $crate::utils::get_timestamp();
        let thread_id = format!("{:?}", std::thread::current().id());
        let msg = format!("[{}] [INFO] [{}] [{}:{}] {}", timestamp, thread_id, file!(), line!(), format!($($arg)*));
        
        if $crate::DEBUG_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            println!("{}", msg);
            $crate::utils::_log_to_debug_file(&msg);
        }
    };
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        let timestamp = $crate::utils::get_timestamp();
        let thread_id = format!("{:?}", std::thread::current().id());
        let msg = format!("[{}] [ERROR] [{}] [{}:{}] {}", timestamp, thread_id, file!(), line!(), format!($($arg)*));
        
        eprintln!("{}", msg);
        $crate::utils::_log_to_err_file(&msg);
        
        if $crate::DEBUG_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
             $crate::utils::_log_to_debug_file(&msg);
        }
    };
}

// this is for to track how much time it took for convertion,
// everybody has some sort of logging, so why not me? :)
pub fn get_timestamp() -> String {
    Local::now().format("%d-%m %H:%M:%S").to_string()
}

// notification but either i would replace it or just get on with it
pub fn notify(title: &str, body: &str, _folder_path: &str) {
    let _ = Notification::new().summary(title)
        .body(body)
        .show();
}

// gotta add some way to sepearte video, audio, image as seperate
pub fn is_media_file(path: &str) -> (bool, Option<String>) {
    let path_obj = Path::new(path);
    if let Some(file_name) = path_obj.file_name().and_then(|n| n.to_str()) {
        if file_name.contains(".conv.")|| file_name.contains(".converting.") {
            return (false, None);
        }
    }

    let media_exts = [
        //video
        "mp4", "mov", "mkv", "avi", "flv", "wmv", "webm", "mpg", "mpeg", "m4v", "ts", "ogv", "rm",
        "3gp", "3g2", "dv", "vob", "mts", "m2ts", "f4v", "f4p", "f4a", "f4b", "gif", "mj2", "nut",
        //audio
        "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "ac3", "alac", "opus", "amr", "au",
        // image
        "jpg", "jpeg", "png", "bmp", "tiff", "tif", "webp", "exr",
    ];

    if let Some(ext) = path_obj.extension() {
        if let Some(ext_str) = ext.to_str() {
            return (
                media_exts.contains(&ext_str.to_lowercase().as_str()),
                Some(ext_str.to_lowercase()),
            );
        }
    }
    (false, None)
}

pub fn get_output(main_path: &str, path: &str, ext: &str) -> Option<String> {
    let path = Path::new(path);
    if let Some(stem) = path.file_stem() {
        if let Some(stem_str) = stem.to_str() {
            let mut out_path = PathBuf::from(main_path);
            let file_name = format!("{}.conv.{}", stem_str, ext);
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
    if !path.exists() {
        return;
    }

    // find matching config
    if let Some(path_config) = config.portal.iter().find(|p| {
        let input_path = Path::new(&p.input_folder);
        if let Ok(canon_input) = input_path.canonicalize() {
            if path.starts_with(&canon_input) {
                return true;
            }
        }
        path.starts_with(input_path)
    }) {
        if is_media_file(&path.to_string_lossy()).0 {
            log!("detected media file: {:?}", path);

            thread::sleep(Duration::from_millis(1500));
            if !path.exists() {
                return;
            }

            if let Some(output_file) = get_output(
                &path_config.output_folder,
                &path.to_string_lossy(),
                &path_config.output_format,
            ) {
                let input_options = string_to_str_slice(&path_config.input_options);
                let output_options = string_to_str_slice(&path_config.output_options);
                media_normal_convert(path, input_options, output_options, &output_file);
            } else {
                err!("failed to determine output file path for {:?}", path);
            }
        }
    }
}

pub fn media_normal_convert(
    input: &Path,
    input_options: Vec<&str>,
    output_options: Vec<&str>,
    output: &str,
) {
    let _input_parent = input.parent().unwrap_or_else(|| Path::new("."));
    let final_output_path = Path::new(output); // This is the final name: video.conv.mp4
    let final_output_dir = final_output_path.parent().unwrap_or_else(|| Path::new("."));

    // determine temp folder path: output_dir/temp/
    let temp_subfolder = final_output_dir.join("temp");
    if !temp_subfolder.exists() {
        if let Err(e) = fs::create_dir_all(&temp_subfolder) {
            err!("Failed to create temporary subfolder {:?}: {:?}", temp_subfolder, e);
            return; 
        }
    }

    // output filename
    let temp_file_stem = final_output_path.file_stem().unwrap_or_default().to_str().unwrap_or("output");
    let target_ext = final_output_path.extension().and_then(|s| s.to_str()).unwrap_or("mp4");
    let temp_output_filename = format!("{}.converting.{}", temp_file_stem, target_ext);
    let temp_output = temp_subfolder.join(&temp_output_filename);

    notify(
        "MPortal: Converting...",
        Path::new(input)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or(""),
        " ",
    );
    log!("converting in temp folder: {:?}", temp_output);

    let mut command = Command::new("ffmpeg");
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }

    let conversion = command
        .args(input_options)
        .arg("-i")
        .arg(input)
        .args(output_options)
        .args(&["-loglevel", "warning", "-y"])
        .arg(&temp_output)
        .status();

    match conversion {
        Ok(status) => {
            if status.success() {
                let final_output_path_for_move = final_output_path.to_path_buf();
                log!("moving finished file to: {:?}", final_output_path_for_move);

                if let Err(e) = fs::rename(&temp_output, &final_output_path_for_move) {
                    err!("failed to move file from temp to final destination: {:?}", e);
                    // note buddy: copy paste have no limitation across file system like moving files.
                    if let Err(e2) = fs::copy(&temp_output, &final_output_path_for_move) {
                        err!("failed to copy file from temp to final destination: {:?}", e2);
                    } else {
                        let _ = fs::remove_file(&temp_output);
                        log!("conversion done: {:?}", final_output_path_for_move);
                        notify(
                            "MPortal: Conversion Done",
                            Path::new(input)
                                .file_name()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or(""),
                            final_output_path_for_move.parent().unwrap().to_str().unwrap(),
                        );
                    }
                } else {
                    log!("conversion done: {:?}", final_output_path_for_move);
                    notify(
                        "MPortal: Conversion Done",
                        Path::new(input)
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or(""),
                        final_output_path_for_move.parent().unwrap().to_str().unwrap(),
                    );
                }
            } else {
                err!("ffmpeg failed for {} with status {:?}", input.display(), status);
                let _ = fs::remove_file(&temp_output);
            }
        }
        Err(e) => {
            err!("failed to execute ffmpeg for {}: {:?}", input.display(), e);
            let _ = fs::remove_file(&temp_output);
        }
    }
}