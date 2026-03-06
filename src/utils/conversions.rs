//! UTF‑16 conversions for interacting with the Windows API.
//!
//! Windows APIs often expect strings as null‑terminated UTF‑16 (wide) strings.
//! This module provides safe conversions between Rust’s UTF‑8 `String`/`str`
//! and the wide strings used by Windows.

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

/// Converts a Rust UTF‑8 string slice into a null‑terminated wide string
/// (`Vec<u16>`) suitable for passing to Windows API functions.
///
/// The returned vector includes a terminating null character (`0`). It can be
/// passed directly to functions expecting `PCWSTR` or `PWSTR` (via `.as_ptr()`).
///
/// # Example
/// ```
/// let wide = to_wide_string("Hello, world!");
/// unsafe {
///     let ptr = wide.as_ptr(); // use as PCWSTR
/// }
/// ```
pub fn to_wide_string(s: &str) -> Vec<u16> {
    OsString::from(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Attempts to convert a null‑terminated wide string (UTF‑16) into a Rust
/// `String`.
///
/// The input is a raw pointer to a null‑terminated sequence of `u16`.
/// If the pointer is null, or if the wide string is not valid UTF‑16,
/// `None` is returned.
///
/// # Safety
/// The caller must ensure that `ptr` points to a valid null‑terminated
/// wide string and that the memory remains valid for the entire length
/// of the string. This function reads from `ptr` until a null word is
/// encountered; it does not check for buffer overruns.
///
/// # Example
/// ```
/// let wide = to_wide_string("Example");
/// let s = unsafe { from_wide_string(wide.as_ptr()) };
/// assert_eq!(s.as_deref(), Some("Example"));
/// ```
pub fn from_wide_string(ptr: *const u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe {    
        // Count characters until null terminator
        let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice).into_string().ok()
   }
}


