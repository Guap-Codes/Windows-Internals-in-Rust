use crate::{Thread, Result};
use std::sync::atomic::{AtomicU32, Ordering};
use owo_colors::OwoColorize; 

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn run_thread_demo() -> Result<()> {
    println!("\n{}", "🧵 Thread Pool Demo".purple().bold());  // bright_purple -> purple
    
    let mut threads = Vec::new();
    
    unsafe extern "system" fn worker_thread(_: *mut std::ffi::c_void) -> u32 {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("    Worker {} executing", id);
        std::thread::sleep(std::time::Duration::from_millis(500));
        0
    }
    
    for i in 0..5 {
        let thread = Thread::create(
            Some(worker_thread),
            Some(std::ptr::null()),
            Default::default()
        )?;
        threads.push((i, thread));
        println!("  {} Spawned thread {}", "→".yellow(), i);
    }
    
    for (i, thread) in threads {
        thread.wait(None)?;
        println!("  {} Thread {} completed", "✓".green(), i);
    }
    
    Ok(())
}


