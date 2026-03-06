use crate::utils::errors::{Result, WindowsError};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT, STILL_ACTIVE},
        System::Threading::{
            CreateThread, GetCurrentThread, GetCurrentThreadId, GetExitCodeThread,
            LPTHREAD_START_ROUTINE, OpenThread, ResumeThread, SuspendThread, TerminateThread, 
            WaitForSingleObject, THREAD_ACCESS_RIGHTS, THREAD_ALL_ACCESS, THREAD_CREATION_FLAGS,
        },
    },
};
use std::ffi::c_void;

/// Represents a Windows Thread - the unit of execution
#[derive(Debug)]
pub struct Thread {
    handle: HANDLE,
    id: u32,
    is_pseudo: bool,
}

impl Thread {

    /// Create a new thread in the current process
    pub fn create(
        start_address: LPTHREAD_START_ROUTINE,
        parameter: Option<*const c_void>,
        creation_flags: THREAD_CREATION_FLAGS,
    ) -> Result<Self> {
        unsafe {
            let mut thread_id = 0u32;
            let handle = CreateThread(
                None,                      // lpThreadAttributes
                0,                         // dwStackSize (0 = default)
                start_address,              // lpStartAddress
                parameter,                  // lpParameter
                creation_flags,              // dwCreationFlags
                Some(&mut thread_id),        // lpThreadId
            )?;

            Self::from_raw(handle, thread_id)
        }
    }

    /// Current executing thread (pseudo-handle)
    pub fn current() -> Self {
        unsafe {
            Self {
                handle: GetCurrentThread(),
                id: GetCurrentThreadId(),
                is_pseudo: true,
            }
        }
    }

    /// Wrap raw handle (takes ownership)
    pub(crate) unsafe fn from_raw(handle: HANDLE, id: u32) -> Result<Self> {
        if handle.is_invalid() {
            return Err(WindowsError::InvalidHandle);
        }
        Ok(Self {
            handle,
            id,
            is_pseudo: false,
        })
    }

    /// Suspend execution (returns previous suspend count)
    pub fn suspend(&self) -> Result<u32> {
        unsafe {
            let count = SuspendThread(self.handle);
            if count == u32::MAX {
                return Err(WindowsError::OperationFailed("Suspend failed".into()));
            }
            Ok(count)
        }
    }

    /// Resume execution (returns previous suspend count)
    pub fn resume(&self) -> Result<u32> {
        unsafe {
            let count = ResumeThread(self.handle);
            if count == u32::MAX {
                return Err(WindowsError::OperationFailed("Resume failed".into()));
            }
            Ok(count)
        }
    }

    /// Wait for thread completion
    pub fn wait(&self, timeout_ms: Option<u32>) -> Result<bool> {
        unsafe {
            let timeout = timeout_ms.unwrap_or(0xFFFFFFFF);
            match WaitForSingleObject(self.handle, timeout) {
                WAIT_OBJECT_0 => Ok(true),
                WAIT_TIMEOUT => Ok(false),
                _ => Err(WindowsError::Timeout),
            }
        }
    }

    /// Get exit code (None = still running, Some(code) = terminated)
    pub fn exit_code(&self) -> Result<Option<u32>> {
        unsafe {
            let mut code = 0u32;
            GetExitCodeThread(self.handle, &mut code)?;
            if code == STILL_ACTIVE.0 as u32 {  // STILL_ACTIVE is a constant of type u32
                Ok(None)
            } else {
                Ok(Some(code))
            }
        }
    }

    /// Terminate the thread forcefully (use only in emergencies)
    pub fn terminate(&self, exit_code: u32) -> Result<()> {
        unsafe {
            TerminateThread(self.handle, exit_code)?;
            Ok(())
        }
    }

    /// Open an existing thread by its ID with specified access rights.
    pub fn open(thread_id: u32, desired_access: THREAD_ACCESS_RIGHTS) -> Result<Self> {
        unsafe {
            let handle = OpenThread(desired_access, false, thread_id)?;
            if handle.is_invalid() {
                return Err(WindowsError::NotFound);
            }
            Self::from_raw(handle, thread_id)
        }
    }

    /// Open with full access.
    pub fn open_all_access(thread_id: u32) -> Result<Self> {
        Self::open(thread_id, THREAD_ALL_ACCESS)
    }

    pub fn id(&self) -> u32 { self.id }
    pub fn handle(&self) -> HANDLE { self.handle }
}

impl Drop for Thread {
    fn drop(&mut self) {
        if !self.is_pseudo {
            unsafe { let _ = CloseHandle(self.handle); }
        }
    }
}
