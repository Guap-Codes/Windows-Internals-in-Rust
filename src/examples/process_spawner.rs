use crate::{Process, Result};
use owo_colors::OwoColorize;  // Changed
use std::path::PathBuf;

pub fn run_spawner_demo() -> Result<()> {
    println!("\n{}", " Process Spawner & Monitor".blue().bold());  // bright_blue -> blue
    
    let apps = vec![
        (r"C:\Windows\System32\calc.exe", "Calculator"),
        (r"C:\Windows\System32\notepad.exe", "Notepad"),
    ];
    
    let mut processes = Vec::new();
    
    for (path, name) in apps {
        if !PathBuf::from(path).exists() {
            println!("  {} {} not found, skipping", "⚠".yellow(), name);
            continue;
        }
        
        let (proc, thread) = Process::create(path, None, false)?;
        println!("  {} Started {} (PID: {})", "→".green(), name, proc.id());
        
        processes.push((name, proc, thread));
    }
    
    println!("  {} Monitoring for 5 seconds...", "⏱".cyan());
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    for (name, proc, _) in &processes {
        match proc.is_alive() {
            Ok(true) => println!("  {} {} still running", "●".green(), name),
            Ok(false) => println!("  {} {} exited with code {:?}", "○".red(), name, proc.exit_code()),
            Err(e) => println!("  {} {} error: {}", "✗".red(), name, e),
        }
    }
    
    for (name, proc, _) in processes {
        if proc.is_alive()? {
            proc.terminate(0)?;
            println!("  {} Terminated {}", "✓".yellow(), name);
        }
    }
    
    Ok(())
}


