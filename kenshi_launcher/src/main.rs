use clap::{Parser, Subcommand};
use dll_syringe::{Syringe, process::OwnedProcess};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "Kenshi Launcher")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Launch {
        #[arg(long, default_value = "kenshi_mod.dll")]
        dll: String,
        #[arg(long, default_value = "kensch.exe")]
        exe: String,
    },
    Inject {
        #[arg(long, default_value = "kenshi_mod.dll")]
        dll: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Launch { dll, exe } => {
            info!("Launching {} with {}...", exe, dll);
            
            let mut child = Command::new(&exe)
                .spawn()
                .expect("Failed to start Kenshi");

            // Give it a moment to initialize
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let pid = child.id().ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;
            info!("Kenshi PID: {}", pid);

            inject_dll(pid, &dll)?;
        }
        Commands::Inject { dll } => {
            // Find process by name (simplified)
            // In real app use sysinfo or similar
            info!("Injecting {} into running process...", dll);
            // This part requires finding the process ID which is OS specific, skipping for this snippet
            // assuming the user will use --launch for now or we implement process finding.
            error!("Injecting into running process not fully implemented yet, use Launch mode.");
        }
    }

    Ok(())
}

fn inject_dll(pid: u32, dll_path: &str) -> anyhow::Result<()> {
    let target = OwnedProcess::from_pid(pid)?;
    let syringe = Syringe::for_process(target);
    
    let path = Path::new(dll_path).canonicalize()?;
    syringe.inject(&path)?;
    
    info!("Successfully injected DLL!");
    Ok(())
}
