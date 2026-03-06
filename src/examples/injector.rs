/// EDUCATIONAL PURPOSE ONLY - Shows memory allocation in remote process
/// and thread resumption.
use crate::{Process, Result};
use owo_colors::OwoColorize;
use std::thread::sleep;
use std::time::Duration;

pub fn demonstrate_remote_memory() -> Result<()> {
    println!("\n{}", "💉 Remote Memory Operations (Educational)".red().bold());
    println!("  {}", "This demonstrates allocation in remote process space".dimmed());

    // Create a suspended Notepad process
    let (target, main_thread) = Process::create(
        r"C:\Windows\System32\notepad.exe",
        None,
        true, // suspended
    )?;

    println!("  {} Target PID: {}", "→".yellow(), target.id());

    unsafe {
        use windows::Win32::System::Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};

        let remote_mem = VirtualAllocEx(
            target.handle(),
            None,
            1024,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_mem.is_null() {
            println!("  {} Allocation failed", "✗".red());
        } else {
            println!("  {} Allocated 1KB at {:p}", "✓".green(), remote_mem);
        }
    }

    // Resume the suspended main thread so Notepad starts running
    println!("  {} Resuming main thread...", "→".yellow());
    main_thread.resume()?;

    // Give Notepad a moment to appear (optional)
    sleep(Duration::from_secs(2));

    // Clean up: terminate the process
    target.terminate(0)?;
    println!("  {} Process terminated", "✓".green());

    Ok(())
}
