//! Windows Job Objects – a kernel feature that allows groups of processes
//! to be managed as a unit. Jobs can enforce limits (CPU, memory, active process count)
//! and provide notifications.

use crate::core::process::Process;
use crate::utils::conversions::to_wide_string;
use crate::utils::errors::Result;
use std::mem::zeroed;
use std::ptr::null_mut;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOB_OBJECT_LIMIT, JOBOBJECT_BASIC_LIMIT_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PRIORITY_CLASS,
    JOB_OBJECT_LIMIT_WORKINGSET,
};

/// A Windows Job Object, capable of containing one or more processes.
///
/// Jobs provide resource controls (limits) and lifetime management. Once a process
/// is assigned to a job, it (and any child processes) remain in that job.
///
/// # Examples
/// ```
/// use windows_internals::core::{JobObject, JobLimits};
///
/// let job = JobObject::create(Some("MySandbox"))?;
/// job.set_limits(&JobLimits {
///     max_working_set: Some(20 * 1024 * 1024), // 20 MB
///     active_process_limit: Some(2),            // max 2 processes
///     kill_on_close: true,
///     ..Default::default()
/// })?;
/// // ... assign processes ...
/// // When `job` goes out of scope, all contained processes are terminated
/// // because `kill_on_close` is true.
/// # Ok::<_, windows_internals::utils::errors::WindowsError>(())
/// ```
#[derive(Debug)]
pub struct JobObject {
    handle: HANDLE,
    name: Option<String>,
}

/// Resource constraints that can be applied to a job.
///
/// Use [`Default::default()`] to start with all limits disabled.
#[derive(Debug, Clone, Default)]
pub struct JobLimits {
    /// Maximum working set size (in bytes) for each process in the job.
    pub max_working_set: Option<usize>,
    /// Minimum working set size (in bytes) for each process in the job.
    pub min_working_set: Option<usize>,
    /// Maximum number of active processes that can be part of the job.
    pub active_process_limit: Option<u32>,
    /// Priority class for all processes in the job (e.g., `NORMAL_PRIORITY_CLASS`).
    pub priority_class: Option<u32>,
    /// If true, all processes in the job are terminated when the last handle to the job is closed.
    pub kill_on_close: bool,
}

impl JobObject {
    /// Creates a new job object, optionally with a name.
    ///
    /// # Arguments
    /// * `name` – If provided, creates a named job that can be opened by other processes.
    ///            Names are case‑insensitive and follow the Windows object namespace rules.
    ///
    /// # Errors
    /// Returns an error if the underlying `CreateJobObjectW` fails (e.g., invalid name or system limit).
    pub fn create(name: Option<&str>) -> Result<Self> {
        unsafe {
            let name_wide = name.map(to_wide_string);
            let name_ptr = name_wide
                .as_ref()
                .map(|v| PCWSTR(v.as_ptr()))
                .unwrap_or(PCWSTR(null_mut()));

            let handle = CreateJobObjectW(None, name_ptr)?;
            // CreateJobObjectW returns NULL on failure, which the `windows` crate maps to an error.
            // So we don't need an explicit `INVALID_HANDLE_VALUE` check.

            Ok(Self {
                handle,
                name: name.map(|s| s.to_string()),
            })
        }
    }

    /// Assigns a process to this job.
    ///
    /// A process can only be in one job at a time. If the process is already in a job,
    /// the function fails with `WindowsError::ApiError` (error code `ERROR_ALREADY_IN_JOB`).
    ///
    /// # Arguments
    /// * `process` – A reference to a [`Process`] that is not yet assigned to any job.
    pub fn assign(&self, process: &Process) -> Result<()> {
        unsafe {
            AssignProcessToJobObject(self.handle, process.handle())?;
            Ok(())
        }
    }

    /// Applies resource limits to the job.
    ///
    /// Limits are enforced for every process currently in the job, as well as any
    /// processes added later. Existing limits can be changed by calling this method again.
    ///
    /// # Arguments
    /// * `limits` – The desired limits; any `None` field leaves the corresponding limit unchanged.
    pub fn set_limits(&self, limits: &JobLimits) -> Result<()> {
        unsafe {
            // Start with zeroed structures – all fields default to 0, which is a valid starting point.
            let mut basic: JOBOBJECT_BASIC_LIMIT_INFORMATION = zeroed();
            let mut extended: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();

            let mut flags = JOB_OBJECT_LIMIT::default(); // all flags cleared

            // Working set limits
            if let Some(max) = limits.max_working_set {
                basic.MaximumWorkingSetSize = max;
                flags |= JOB_OBJECT_LIMIT_WORKINGSET;
            }
            if let Some(min) = limits.min_working_set {
                basic.MinimumWorkingSetSize = min;
                flags |= JOB_OBJECT_LIMIT_WORKINGSET;
            }

            // Active process limit
            if let Some(limit) = limits.active_process_limit {
                basic.ActiveProcessLimit = limit;
                flags |= JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
            }

            // Priority class
            if let Some(priority) = limits.priority_class {
                basic.PriorityClass = priority;
                flags |= JOB_OBJECT_LIMIT_PRIORITY_CLASS;
            }

            // Kill on close
            if limits.kill_on_close {
                flags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            }

            basic.LimitFlags = flags;
            extended.BasicLimitInformation = basic;

            SetInformationJobObject(
                self.handle,
                JobObjectExtendedLimitInformation,
                &extended as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )?;

            Ok(())
        }
    }

    /// Returns the name of the job, if it was created with one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the raw Windows handle of the job.
    /// Use with caution – the handle remains owned by the `JobObject`.
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for JobObject {
    /// Closes the job handle. If the job was created with `kill_on_close` and this is the
    /// last open handle to the job, all processes in the job are terminated.
    fn drop(&mut self) {
        unsafe {
            // Ignore errors – nothing we can do.
            let _ = CloseHandle(self.handle);
        }
    }
}


