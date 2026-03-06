//! Example implementations demonstrating Windows internals concepts
//! 
//! Each module showcases different aspects of Process, Thread, and Job management.
//! These are educational implementations showing real-world usage patterns.

pub mod sandbox;
pub mod thread_pool;
pub mod process_spawner;
pub mod injector;

// Re-export demo functions
pub use sandbox::run_sandbox_demo;
pub use thread_pool::run_thread_demo;
pub use process_spawner::run_spawner_demo;
pub use injector::demonstrate_remote_memory;
