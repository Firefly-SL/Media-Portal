use clap::{Parser, ArgAction};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

#[derive(Parser)]
#[command(name = "mportal-cli", disable_version_flag=true, about = "MPortal CLI Controller", version)]
struct CliFlags {
    #[arg(short = 'v', long = "version",  help = "Print version", action = ArgAction::Version)]
    version: (),
    
    #[arg(short = 'd', long = "debug",  help = "Start daemon with live debug logs", action = ArgAction::SetTrue)]
    debug: bool,
}

fn main() {
    let cli_flags = CliFlags::parse();
    // Determine daemon executable path
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");
    let current_dir = current_exe.parent().expect("Failed to get parent dir");
    
    #[cfg(windows)]
    let daemon_name = "mportal-daemon.exe";
    #[cfg(not(windows))]
    let daemon_name = "mportal-daemon";
    
    let daemon_path = current_dir.join(daemon_name);

    if !daemon_path.exists() {
        eprintln!("Error: Daemon executable not found at {:?}", daemon_path);
        std::process::exit(1);
    }

    if cli_flags.debug {
        println!("Starting daemon in debug mode...");
        
        let _child = Command::new(&daemon_path)
            .arg("-d")
            .stdout(Stdio::null()) // redirect both so the logs won't become double,
            .stderr(Stdio::null()) // now onlt logs readed by the cli work, 
                                   // i didn't try this way and i believe the way i have is much better and predictable on windows.
            .spawn()
            .expect("Failed to start daemon");
            
        println!("Daemon started. Swimming through debug log...");
        
        // the log file
        let log_path = std::env::temp_dir().join("mportal_debug.log");
        
        // waiting for the log file
        let mut retries = 0;
        while !log_path.exists() {
            if retries > 20 {
                eprintln!("Timeout waiting for log file: {:?}", log_path);
                return;
            }
            thread::sleep(Duration::from_millis(500));
            retries += 1;
        }

        tail_file(&log_path);
    } else {
        println!("Starting daemon...");
        match Command::new(&daemon_path)
            .stdout(Stdio::null()) // in either way, i don't need logs now so, yeah
            .stderr(Stdio::null())
            .spawn() {
            Ok(_) => println!("Daemon started successfully."),
            Err(e) => eprintln!("Failed to start daemon: {:?}", e),
        }
    }
}

fn tail_file(path: &Path) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file {:?}: {:?}", path, e);
            return;
        }
    };

    let _ = file.seek(SeekFrom::Start(0));

    let mut buffer = [0; 1024];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => {
                // end of file, wait..
                thread::sleep(Duration::from_millis(100));
            }
            Ok(n) => {
                print!("{}", String::from_utf8_lossy(&buffer[..n]));
            }
            Err(e) => {
                eprintln!("Error reading log {:?}: {:?}", path, e);
                break;
            }
        }
    }
}
