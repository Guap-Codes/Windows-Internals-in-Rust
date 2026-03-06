//! Core Windows internals implementations
//! 
//! This module provides safe wrappers around Windows kernel objects:
//! - Processes: Resource containers
//! - Threads: Execution units  
//! - Job Objects: Process containers with resource governance

pub mod process;
pub mod thread;
pub mod job;

// Re-export main types for convenience
pub use process::Process;
pub use thread::Thread;
pub use job::{JobObject, JobLimits};
