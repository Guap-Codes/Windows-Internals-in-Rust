use crate::{Process, JobObject, Result};
use owo_colors::OwoColorize;  // Changed from colored::Colorize
use std::time::Duration;

pub fn run_sandbox_demo() -> Result<()> {
    println!("{}", "🔒 Creating Process Sandbox".cyan().bold());  // bright_cyan -> cyan
    
    let sandbox = JobObject::create(Some("RustSandbox"))?;
    
    let limits = crate::core::job::JobLimits {
        max_working_set: Some(50 * 1024 * 1024),
        active_process_limit: Some(5),
        kill_on_close: true,
        ..Default::default()
    };
    
    sandbox.set_limits(&limits)?;
    println!("  {} Resource limits applied", "✓".green());

    let (proc, thread) = Process::create(
        r"C:\Windows\System32\notepad.exe",
        None,
        true
    )?;
    
    sandbox.assign(&proc)?;
    println!("  {} Process {} assigned to sandbox", "✓".green(), proc.id());

    thread.resume()?;
    println!("  {} Process running (will auto-kill in 3s)", "▶".green());  // bright_green -> green
    
    std::thread::sleep(Duration::from_secs(3));
    
    println!("  {} Sandbox closed, processes terminated", "✓".green());
    
    Ok(())
}

