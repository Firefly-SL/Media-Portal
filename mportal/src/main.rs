use clap::{Parser, ArgAction, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "MPortal", version)]
struct Cli {
    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    version: (),
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Update,
    Uninstall,
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Update) | Some(Commands::Install)  = cli.command {
        println!("Updating...");
        
        let script_path = "../../installer/installer.ps1";

        let status = Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-File", script_path])
            .status()
            .expect("failed to run powershell script.");

        if !status.success() {
            eprintln!("Update script failed.");
        }
    } else if let Some(Commands::Uninstall) = cli.command {
        println!("Uninstalling...")
    }
}
