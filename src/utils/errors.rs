//! Custom error types for Windows internals operations.
//!
//! This module defines a unified error enum [`WindowsError`] that wraps
//! Windows API errors and adds higher‑level error kinds. The [`Result`] alias
//! simplifies return types throughout the crate.

use thiserror::Error;
use windows::core::Error as WinError;

/// The error type for all Windows internals operations.
///
/// It captures both low‑level Windows API errors (converted automatically
/// via `From<windows::core::Error>`) and higher‑level errors that arise
/// from invalid input or unexpected conditions.
#[derive(Error, Debug)]
pub enum WindowsError {
    /// An error returned directly by a Windows API function.
    ///
    /// The `windows::core::Error` contains the HRESULT or Win32 error code
    /// and a message if available.
    #[error("Windows API error: {0}")]
    ApiError(#[from] WinError),

    /// The handle provided to an operation was invalid.
    ///
    /// This can happen when a handle is null, closed, or does not refer
    /// to a valid kernel object.
    #[error("Invalid handle provided")]
    InvalidHandle,

    /// A general operation failure with a custom message.
    ///
    /// Used when the operation failed for a reason that doesn't fit the
    /// other variants, or when we want to attach a specific explanation.
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Access was denied due to insufficient privileges or rights.
    ///
    /// Typically corresponds to `ERROR_ACCESS_DENIED`.
    #[error("Access denied - insufficient privileges")]
    AccessDenied,

    /// The requested process or thread could not be found.
    ///
    /// Typically corresponds to `ERROR_NOT_FOUND` or `ERROR_INVALID_PARAMETER`
    /// when the identifier does not exist.
    #[error("Process or thread not found")]
    NotFound,

    /// A wait operation timed out before the object was signaled.
    #[error("Timeout occurred")]
    Timeout,

    /// An invalid parameter was supplied to a function.
    ///
    /// The attached string provides details about which parameter was invalid.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// A system resource limit was exceeded (e.g., too many processes,
    /// memory quota reached).
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
}

/// A specialized `Result` type for Windows internals operations.
///
/// This alias makes it convenient to return [`WindowsError`] throughout
/// the crate without spelling out the full type.
pub type Result<T> = std::result::Result<T, WindowsError>;

