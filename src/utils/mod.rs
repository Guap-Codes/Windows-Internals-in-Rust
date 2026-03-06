//! Utility modules for Windows operations
//! 
//! Provides error handling, string conversions, and helper functions
//! for working with Windows APIs in Rust.

pub mod errors;
pub mod conversions;

// Re-export commonly used items
pub use errors::{WindowsError, Result};
pub use conversions::{to_wide_string, from_wide_string};
