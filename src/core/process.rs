//! Windows Process Management – Create, open, and control processes.
//!
//! This module provides a safe Rust wrapper around the Windows process API,
//! handling handle lifetime and common operations. It includes a flexible
//! builder for process creation with full control over flags, attributes,
//! and environment.

use crate::core::thread::Thread;
use crate::utils::conversions::to_wide_string;
use crate::utils::errors::{Result, WindowsError};
use std::mem::zeroed;
use std::ptr::null_mut;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, STILL_ACTIVE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, GetCurrentProcessId, GetExitCodeProcess, OpenProcess,
    TerminateProcess, WaitForSingleObject, CREATE_NEW_CONSOLE, CREATE_SUSPENDED,
    DETACHED_PROCESS, PROCESS_ACCESS_RIGHTS, PROCESS_ALL_ACCESS, PROCESS_CREATION_FLAGS,
    PROCESS_INFORMATION, STARTUPINFOW,
};

/// A safe wrapper around a Windows process handle.
///
/// `Process` represents either a newly created process or an existing one opened by PID.
/// It automatically closes the handle when dropped (except for pseudo‑handles like `current()`).
///
/// # Examples
/// ```
/// use windows_internals::core::Process;
///
/// // Create a new Notepad process, suspended
/// let (proc, main_thread) = Process::create(r"C:\Windows\System32\notepad.exe", None, true)?;
/// println!("Created process with PID: {}", proc.id());
///
/// // Wait for it to exit (it's suspended, so this would time out – handle carefully)
/// // proc.wait(Some(5000))?;
///
/// // Terminate it
/// proc.terminate(0)?;
/// # Ok::<_, windows_internals::utils::errors::WindowsError>(())
/// ```
#[derive(Debug)]
pub struct Process {
    handle: HANDLE,
    id: u32,
    is_pseudo: bool, // true for handles that should not be closed (e.g., GetCurrentProcess)
}

impl Process {
    // ---------- Factory methods (simple) ----------

    /// Returns a `Process` representing the current process.
    ///
    /// The handle returned by `GetCurrentProcess` is a pseudo‑handle; it does not need to be
    /// closed and is not inherited by child processes. This method does not allocate any system
    /// resources.
    pub fn current() -> Self {
        unsafe {
            Self {
                handle: GetCurrentProcess(),
                id: GetCurrentProcessId(),
                is_pseudo: true,
            }
        }
    }

    /// Opens an existing process by its process ID (PID) with the specified access rights.
    ///
    /// # Arguments
    /// * `pid` – The process identifier of the target process.
    /// * `access` – Desired access rights (e.g., `PROCESS_ALL_ACCESS`, `PROCESS_TERMINATE`).
    ///
    /// # Errors
    /// Returns:
    /// * `WindowsError::NotFound` if the process does not exist or access is denied.
    /// * `WindowsError::ApiError` for other system errors (e.g., invalid parameter).
    pub fn open(pid: u32, access: PROCESS_ACCESS_RIGHTS) -> Result<Self> {
        unsafe {
            let handle = OpenProcess(access, false, pid)?;

            // OpenProcess returns NULL on failure, which the `windows` crate maps to an error.
            // However, if it succeeds but the handle is invalid (shouldn't happen), we still check.
            if handle.is_invalid() {
                return Err(WindowsError::NotFound);
            }

            Ok(Self {
                handle,
                id: pid,
                is_pseudo: false,
            })
        }
    }

    /// Opens a process with full access (`PROCESS_ALL_ACCESS`).
    ///
    /// Equivalent to `Process::open(pid, PROCESS_ALL_ACCESS)`.
    pub fn open_all_access(pid: u32) -> Result<Self> {
        Self::open(pid, PROCESS_ALL_ACCESS)
    }

    /// Creates a new process from the given executable path with basic options.
    ///
    /// This is a convenience wrapper around [`ProcessBuilder`]. For advanced control
    /// (creation flags, security attributes, stack size, etc.), use the builder directly.
    ///
    /// # Arguments
    /// * `app_path` – The path to the executable (UTF‑8 encoded). Must exist and be accessible.
    /// * `args` – Optional command‑line arguments (will be appended after the executable name).
    /// * `suspended` – If `true`, the process is created in a suspended state; its main thread
    ///                 must be resumed with `Thread::resume()` before it runs.
    ///
    /// # Returns
    /// On success, a tuple containing the `Process` object and its main `Thread`.
    ///
    /// # Errors
    /// Returns an error if the executable cannot be found, if the application path is malformed,
    /// or if any other system error occurs during creation.
    pub fn create(
        app_path: &str,
        args: Option<&str>,
        suspended: bool,
    ) -> Result<(Self, Thread)> {
        let builder = ProcessBuilder::new(app_path);
        let builder = if let Some(cmd) = args {
            builder.args(cmd)
        } else {
            builder
        };
        let builder = if suspended {
            builder.suspended()
        } else {
            builder
        };
        builder.spawn()
    }

   // ---------- Process operations ----------

    /// Terminates the process with the given exit code.
    ///
    /// This is a forceful termination; the process does not have a chance to clean up.
    /// Use only when necessary (e.g., the process is unresponsive). For normal termination,
    /// consider waiting for the process to exit on its own.
    ///
    /// # Arguments
    /// * `exit_code` – The exit code that the process will report (e.g., 0 for success).
    pub fn terminate(&self, exit_code: u32) -> Result<()> {
        unsafe {
            TerminateProcess(self.handle, exit_code)?;
            Ok(())
        }
    }

    /// Waits for the process to exit.
    ///
    /// # Arguments
    /// * `timeout_ms` – Maximum time to wait in milliseconds, or `None` to wait indefinitely.
    ///
    /// # Returns
    /// * `Ok(true)` if the process exited within the timeout.
    /// * `Ok(false)` if the timeout expired.
    /// * `Err(...)` if the wait failed (e.g., invalid handle).
    pub fn wait(&self, timeout_ms: Option<u32>) -> Result<bool> {
        unsafe {
            let timeout = timeout_ms.unwrap_or(0xFFFFFFFF); // INFINITE
            let result = WaitForSingleObject(self.handle, timeout);

            match result {
                WAIT_OBJECT_0 => Ok(true),
                WAIT_TIMEOUT => Ok(false),
                _ => Err(WindowsError::Timeout), // Should not happen, but handle generically.
            }
        }
    }

    /// Waits for the process to exit and returns its exit code.
    ///
    /// This is equivalent to calling `wait(None)` followed by `exit_code()`, but combined
    /// into a single operation for convenience.
    ///
    /// # Returns
    /// * `Ok(code)` – The exit code of the process after it has terminated.
    /// * `Err(...)` – If the wait or exit code retrieval fails.
    pub fn wait_and_exit_code(&self) -> Result<u32> {
        self.wait(None)?;
        match self.exit_code()? {
            Some(code) => Ok(code),
            None => Err(WindowsError::OperationFailed(
                "Process exited but exit code not available".into(),
            )),
        }
    }

    /// Retrieves the exit code of the process.
    ///
    /// # Returns
    /// * `Ok(None)` if the process is still running.
    /// * `Ok(Some(code))` if the process has terminated.
    /// * `Err(...)` if the query failed.
    pub fn exit_code(&self) -> Result<Option<u32>> {
        unsafe {
            let mut code: u32 = 0;
            GetExitCodeProcess(self.handle, &mut code)?;

            if code == STILL_ACTIVE.0 as u32 {
                Ok(None)
            } else {
                Ok(Some(code))
            }
        }
    }

    /// Checks whether the process is still alive.
    ///
    /// Equivalent to `self.exit_code()?.is_none()`.
    pub fn is_alive(&self) -> Result<bool> {
        Ok(self.exit_code()?.is_none())
    }

    // ---------- Accessors ----------

    /// Returns the process ID (PID).
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the raw Windows handle.
    ///
    /// The handle remains owned by the `Process` and should not be closed manually.
    /// It can be used in other Windows API calls that require a process handle.
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for Process {
    /// Closes the process handle. If this is the last open handle to the process,
    /// the process object is released from the system (but the process itself may
    /// still be running). Pseudo‑handles are never closed.
    fn drop(&mut self) {
        if !self.is_pseudo {
            unsafe {
                // Ignore errors – nothing we can do.
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ProcessBuilder – flexible process creation
// -----------------------------------------------------------------------------

/// A builder for creating Windows processes with fine‑grained control.
///
/// # Examples
/// ```
/// use windows_internals::core::ProcessBuilder;
///
/// let (proc, thread) = ProcessBuilder::new(r"C:\Windows\System32\notepad.exe")
///     .args("readme.txt")
///     .new_console()
///     .current_directory(r"C:\Users\Public")
///     .spawn()?;
/// # Ok::<_, windows_internals::utils::errors::WindowsError>(())
/// ```
#[derive(Debug, Default)]
pub struct ProcessBuilder {
    application: Option<String>,          // lpApplicationName
    command_line: Option<String>,         // stored to keep the wide string alive
    creation_flags: PROCESS_CREATION_FLAGS,
    inherit_handles: bool,
    process_attributes: Option<SECURITY_ATTRIBUTES>,
    thread_attributes: Option<SECURITY_ATTRIBUTES>,
    current_directory: Option<String>,
    // Environment block support could be added later.
}

impl ProcessBuilder {
    /// Creates a new builder for the given executable.
    ///
    /// # Arguments
    /// * `app_path` – Path to the executable (UTF‑8). May be quoted if it contains spaces.
    pub fn new(app_path: impl Into<String>) -> Self {
        Self {
            application: Some(app_path.into()),
            ..Default::default()
        }
    }

    /// Sets the full command line (including the executable name and all arguments).
    ///
    /// If you only need to add arguments, use `.args()` instead. Providing a command line
    /// overrides the application name for the `lpCommandLine` parameter of `CreateProcessW`.
    pub fn command_line(mut self, cmd: impl Into<String>) -> Self {
        self.command_line = Some(cmd.into());
        self
    }

    /// Appends arguments to the executable. If no command line was set, this builds one
    /// from the application path followed by the arguments. The executable path is quoted
    /// automatically if it contains spaces.
    ///
    /// This method can be called multiple times; arguments are concatenated with spaces.
    pub fn args(mut self, args: impl AsRef<str>) -> Self {
        let args = args.as_ref();
        if let Some(existing) = self.command_line.as_mut() {
            existing.push(' ');
            existing.push_str(args);
        } else {
            // Start building from the application path (quote if needed)
            let app = self.application.as_ref().expect("application path not set");
            let quoted_app = if app.contains(' ') {
                format!("\"{}\"", app)
            } else {
                app.to_string()
            };
            self.command_line = Some(format!("{} {}", quoted_app, args));
        }
        self
    }

    /// Adds the `CREATE_SUSPENDED` flag – the process is created with its main thread suspended.
    pub fn suspended(mut self) -> Self {
        self.creation_flags |= CREATE_SUSPENDED;
        self
    }

    /// Adds the `CREATE_NEW_CONSOLE` flag – the new process has its own console.
    pub fn new_console(mut self) -> Self {
        self.creation_flags |= CREATE_NEW_CONSOLE;
        self
    }

    /// Adds the `DETACHED_PROCESS` flag – the process is created without a console.
    pub fn detached(mut self) -> Self {
        self.creation_flags |= DETACHED_PROCESS;
        self
    }

    /// Sets arbitrary creation flags. This overrides any previously set convenience flags.
    pub fn creation_flags(mut self, flags: PROCESS_CREATION_FLAGS) -> Self {
        self.creation_flags = flags;
        self
    }

    /// Sets whether handles are inherited by the new process.
    pub fn inherit_handles(mut self, inherit: bool) -> Self {
        self.inherit_handles = inherit;
        self
    }

    /// Sets security attributes for the process handle.
    pub fn process_attributes(mut self, attrs: SECURITY_ATTRIBUTES) -> Self {
        self.process_attributes = Some(attrs);
        self
    }

    /// Sets security attributes for the thread handle.
    pub fn thread_attributes(mut self, attrs: SECURITY_ATTRIBUTES) -> Self {
        self.thread_attributes = Some(attrs);
        self
    }

    /// Sets the current directory for the new process.
    pub fn current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = Some(dir.into());
        self
    }

    /// Spawns the process with the configured options.
    ///
    /// # Returns
    /// A tuple `(Process, Thread)` representing the new process and its main thread.
    ///
    /// # Errors
    /// Returns an error if the underlying `CreateProcessW` call fails.
    pub fn spawn(self) -> Result<(Process, Thread)> {
        unsafe {
            // Prepare STARTUPINFOW
            let mut startup_info: STARTUPINFOW = zeroed();
            startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

            let mut process_info: PROCESS_INFORMATION = zeroed();

            // Convert strings to wide and keep them alive.
            let app_wide = self.application.as_ref().map(|s| to_wide_string(s));
            let cmd_wide = self.command_line.as_ref().map(|s| to_wide_string(s));

            let app_ptr = app_wide.as_ref().map(|w| PCWSTR(w.as_ptr())).unwrap_or(PCWSTR(null_mut()));
            let cmd_ptr = cmd_wide.as_ref().map(|w| PWSTR(w.as_ptr() as _)); // Note: CreateProcessW expects mutable pointer, but if we pass a string literal it's okay; we are passing a pointer to our own buffer.

            // Convert optional attributes to raw pointers.
            let process_attrs_ptr = self.process_attributes
                .as_ref()
                .map(|a| a as *const SECURITY_ATTRIBUTES)
                .unwrap_or(null_mut());
            let thread_attrs_ptr = self.thread_attributes
                .as_ref()
                .map(|a| a as *const SECURITY_ATTRIBUTES)
                .unwrap_or(null_mut());

            // Current directory wide string.
            let dir_wide = self.current_directory.as_ref().map(|s| to_wide_string(s));
            let dir_ptr = dir_wide.as_ref().map(|w| PCWSTR(w.as_ptr())).unwrap_or(PCWSTR(null_mut()));

            // Note: lpEnvironment is left as None (uses parent's environment). Could be extended.

            CreateProcessW(
                app_ptr,
                cmd_ptr,
                if process_attrs_ptr.is_null() { None } else { Some(process_attrs_ptr) },
                if thread_attrs_ptr.is_null() { None } else { Some(thread_attrs_ptr) },
                self.inherit_handles,
                self.creation_flags,
                None, // environment block
                dir_ptr,   // ← just pass the PCWSTR directly
                &startup_info,
                &mut process_info,
            )?;

            let process = Process {
                handle: process_info.hProcess,
                id: process_info.dwProcessId,
                is_pseudo: false,
            };

            let thread = Thread::from_raw(process_info.hThread, process_info.dwThreadId)?;

            Ok((process, thread))
        }
    }
}

