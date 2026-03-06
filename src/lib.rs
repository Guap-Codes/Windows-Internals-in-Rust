//! # Windows Internals in Rust
//! 
//! A safe, ergonomic Rust wrapper around Windows Process, Thread, and Job APIs.
//! 
//! ## Architecture
//! 
//! - **Process**: Container of resources (memory, handles, security context)
//! - **Thread**: Unit of execution scheduled by the Windows kernel
//! - **Job Object**: Container for multiple processes with resource governance

pub mod core;
pub mod utils;
pub mod examples;

pub use core::{process::Process, thread::Thread, job::JobObject};
pub use utils::errors::{WindowsError, Result};
