use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;  
use windows_internals::examples::{run_sandbox_demo, run_thread_demo, run_spawner_demo, demonstrate_remote_memory};

#[derive(Parser)]
#[command(name = "windows-internals")]
#[command(about = "Windows Process/Thread/Job exploration tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sandbox,
    Threads,
    Spawn,
    Inject,
    All,
}

fn main() -> anyhow::Result<()> {
    println!("{}", r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║     Windows Internals - Rust Edition                      ║
    ║     Processes • Threads • Jobs                            ║
    ╚═══════════════════════════════════════════════════════════╝
    "#.cyan());  // bright_cyan -> cyan

    let cli = Cli::parse();

    match cli.command {
        Commands::Sandbox => run_sandbox_demo()?,
        Commands::Threads => run_thread_demo()?,
        Commands::Spawn => run_spawner_demo()?,
        Commands::Inject => demonstrate_remote_memory()?,
        Commands::All => {
            run_sandbox_demo()?;
            run_thread_demo()?;
            run_spawner_demo()?;
            demonstrate_remote_memory()?;
        }
    }

    Ok(())
}


